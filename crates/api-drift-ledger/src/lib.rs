//! Embedded contract ledger: `ledger!` test + `ledger.jsonl` history.
//!
//! Design: `api-drift/docs/DESIGN-embedded-ledger.md`. An upstream repo adds
//! this crate as a dev-dependency and embeds the ledger as one test:
//!
//! ```ignore
//! // tests/api_drift.rs
//! use api_drift_ledger::{enum_surface, ledger};
//!
//! ledger! {
//!     dir: "api-drift",
//!     surfaces: [
//!         enum_surface!("backend-events", bro_tui::backend::BackendEvent;
//!             Output, Disconnected),
//!     ],
//! }
//! ```
//!
//! Generated test behaviour (`check_all`):
//!
//! - Regenerate each surface; diff against the committed
//!   `<dir>/<surface>.snapshot.json` (snapshot file format v1, fail-closed
//!   on hash mismatch).
//! - No diff → pass.
//! - Diff, and `ledger.jsonl` already records the new content hash → pass.
//! - Diff, unrecorded → **fail** with the classified breaks and the exact
//!   recording command.
//! - `API_DRIFT_UPDATE=1 cargo test` writes the new snapshot and appends a
//!   ledger entry (empty note) for local iteration; CI runs without it.
//!
//! The workflow mirrors evorium's canopy cycle: diff (plan) → verify →
//! append (commit). Corrections append, never rewrite (append-only JSONL).

pub use api_drift::surface::{EnumSurface, Surface};
pub use api_drift::{ApiSnapshot, ClassifiedBreak, SnapshotFileError};

use api_drift::classify::classify_diff;
use api_drift::diff::diff_snapshots;
use api_drift::{parse_snapshot_file, render_snapshot_file, to_string_pretty};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// One recorded contract change (append-only, one JSON object per line).
/// Carries only failure patterns (paths, sigs, hashes) — never user data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Unix seconds when recorded (no chrono dep; ordering + audit).
    pub ts: u64,
    /// Ledger key, e.g. `acp-contract`.
    pub surface: String,
    /// `sha256:` of the previous committed snapshot (empty on first record).
    pub from: String,
    /// `sha256:` of the newly recorded snapshot.
    pub to: String,
    /// Version label at record time (e.g. `1.3.0`).
    pub version: String,
    /// Classified breaks at record time.
    pub breaks: Vec<ClassifiedBreak>,
    /// The migration note — the antibody (`docs/DESIGN` § Evorium borrows).
    pub note: String,
    /// Who recorded it (agent or human).
    pub author: String,
    /// Reference (ticket id), e.g. `BRO-98`.
    #[serde(rename = "ref")]
    pub reference: String,
}

/// Ledger I/O errors.
#[derive(Debug)]
pub enum LedgerError {
    /// Snapshot file parse/verify failure.
    Snapshot(SnapshotFileError),
    /// Ledger file I/O or JSON failure.
    Io(std::io::Error),
    /// Line is not valid JSON (fail closed, do not skip).
    BadLine(usize, serde_json::Error),
    /// JSON serialization failure on write.
    Json(serde_json::Error),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(e) => write!(f, "snapshot: {e}"),
            Self::Io(e) => write!(f, "ledger io: {e}"),
            Self::BadLine(n, e) => write!(f, "ledger line {n}: {e}"),
            Self::Json(e) => write!(f, "ledger json: {e}"),
        }
    }
}

impl From<std::io::Error> for LedgerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<SnapshotFileError> for LedgerError {
    fn from(e: SnapshotFileError) -> Self {
        Self::Snapshot(e)
    }
}

impl From<serde_json::Error> for LedgerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Path of the ledger file for `dir`.
pub fn ledger_path(dir: &Path) -> PathBuf {
    dir.join("ledger.jsonl")
}

/// Read every ledger entry, in append order. Missing file = empty history.
pub fn read_entries(dir: &Path) -> Result<Vec<LedgerEntry>, LedgerError> {
    let path = ledger_path(dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: LedgerEntry =
            serde_json::from_str(line).map_err(|e| LedgerError::BadLine(i + 1, e))?;
        out.push(entry);
    }
    Ok(out)
}

/// True when some entry for `surface` already records `to_hash`.
pub fn is_recorded(dir: &Path, surface: &str, to_hash: &str) -> Result<bool, LedgerError> {
    Ok(read_entries(dir)?
        .iter()
        .any(|e| e.surface == surface && e.to == to_hash))
}

/// How many entries corroborate a given new-hash for `surface` (APIDRIFT-12:
/// the antibody's corroboration counter — rises as more confirm the same
/// break).
pub fn corroboration(dir: &Path, surface: &str, to_hash: &str) -> Result<u32, LedgerError> {
    Ok(read_entries(dir)?
        .iter()
        .filter(|e| e.surface == surface && e.to == to_hash)
        .count()
        .try_into()
        .unwrap_or(u32::MAX))
}

/// Append one entry (creates the file). History is append-only; corrections
/// are new entries, never edits.
pub fn append_entry(dir: &Path, entry: &LedgerEntry) -> Result<(), LedgerError> {
    use std::io::Write;

    std::fs::create_dir_all(dir)?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path(dir))?;
    writeln!(f, "{}", serde_json::to_string(entry)?)?;
    Ok(())
}

fn unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
/// Outcome of checking one surface.
#[derive(Debug)]
pub enum CheckOutcome {
    /// Regenerated surface matches the committed snapshot.
    Clean,
    /// Diffed, but the new hash is already recorded in the ledger.
    Recorded,
    /// Diffed, unrecorded — the test fails with these breaks.
    Unrecorded { breaks: Vec<ClassifiedBreak> },
    /// `API_DRIFT_UPDATE=1`: new snapshot written + entry appended.
    Updated { breaks: Vec<ClassifiedBreak> },
    /// First record: no committed snapshot existed; wrote one.
    Initialized,
}

fn committed_file(dir: &Path, surface: &dyn Surface) -> PathBuf {
    dir.join(format!("{}.snapshot.json", surface.name()))
}

fn load_committed(
    dir: &Path,
    surface: &dyn Surface,
) -> Result<Option<api_drift::SnapshotFile>, LedgerError> {
    let path = committed_file(dir, surface);
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(parse_snapshot_file(&std::fs::read_to_string(path)?)?))
}

/// Record the current shape of `surface`: write the snapshot file and append
/// a ledger entry. Empty `note` = local iteration (`API_DRIFT_UPDATE=1`).
pub fn record(
    dir: &Path,
    surface: &dyn Surface,
    note: &str,
    author: &str,
    reference: &str,
) -> Result<CheckOutcome, LedgerError> {
    let new_snap = surface.snapshot();
    let new_file = render_snapshot_file(surface.name(), &new_snap);
    let committed = load_committed(dir, surface)?;
    let (old_hash, breaks) = match committed {
        None => (String::new(), Vec::new()),
        Some(old) => {
            let old_snap = ApiSnapshot::new(&old.version, old.items);
            (
                old.content_hash,
                classify_diff(&diff_snapshots(&old_snap, &new_snap)),
            )
        }
    };
    std::fs::create_dir_all(dir)?;
    std::fs::write(committed_file(dir, surface), to_string_pretty(&new_file)?)?;
    append_entry(
        dir,
        &LedgerEntry {
            ts: unix_secs(),
            surface: surface.name().to_owned(),
            from: old_hash,
            to: new_file.content_hash,
            version: new_snap.version.clone(),
            breaks: breaks.clone(),
            note: note.to_owned(),
            author: author.to_owned(),
            reference: reference.to_owned(),
        },
    )?;
    Ok(CheckOutcome::Updated { breaks })
}

/// Check one surface against the committed ledger (crate docs decision tree).
/// `update` = `API_DRIFT_UPDATE=1`.
pub fn check(dir: &Path, surface: &dyn Surface, update: bool) -> Result<CheckOutcome, LedgerError> {
    let Some(committed) = load_committed(dir, surface)? else {
        if update {
            record(dir, surface, "", "unknown", "")?;
            return Ok(CheckOutcome::Initialized);
        }
        return Err(LedgerError::Io(std::io::Error::other(format!(
            "no committed snapshot for `{}`; run `API_DRIFT_UPDATE=1 cargo test`",
            surface.name()
        ))));
    };
    let new_snap = surface.snapshot();
    let new_file = render_snapshot_file(surface.name(), &new_snap);
    if committed.items == new_file.items && committed.version == new_file.version {
        return Ok(CheckOutcome::Clean);
    }
    if is_recorded(dir, surface.name(), &new_file.content_hash)? {
        return Ok(CheckOutcome::Recorded);
    }
    let old_snap = ApiSnapshot::new(&committed.version, committed.items);
    Ok(CheckOutcome::Unrecorded {
        breaks: classify_diff(&diff_snapshots(&old_snap, &new_snap)),
    })
}

/// The `ledger!` test body: check every surface, panic with the exact
/// recording command on an unrecorded diff.
pub fn check_all(dir: &Path, surfaces: &[&dyn Surface]) {
    let update = std::env::var("API_DRIFT_UPDATE").is_ok_and(|v| v == "1");
    for surface in surfaces {
        run(dir, *surface, update);
    }
}

/// Check a single surface and panic on failure. Exposed so [`ledger!`]'s
/// expansion calls it per surface (avoids a `Vec<Box<dyn Surface>>` in the
/// generated test).
#[doc(hidden)]
pub fn check_one<S: Surface>(dir: &Path, surface: &S, update: bool) {
    run(dir, surface, update);
}

fn run(dir: &Path, surface: &dyn Surface, update: bool) {
    let outcome = check(dir, surface, update)
        .unwrap_or_else(|e| panic!("api-drift ledger `{}`: {e}", surface.name()));
    match outcome {
        CheckOutcome::Clean => {}
        CheckOutcome::Recorded => {
            println!(
                "api-drift ledger `{}`: diff recorded in ledger.jsonl",
                surface.name()
            );
        }
        CheckOutcome::Initialized => {
            println!(
                "api-drift ledger `{}`: first snapshot committed (API_DRIFT_UPDATE=1)",
                surface.name()
            );
        }
        CheckOutcome::Updated { breaks } => {
            println!(
                "api-drift ledger `{}`: recorded {} break(s) with empty note — replace with a real migration note",
                surface.name(),
                breaks.len()
            );
        }
        CheckOutcome::Unrecorded { breaks } => {
            let mut msg = format!(
                "api-drift ledger `{}`: unrecorded API drift ({} break(s))\n\
                 record it: API_DRIFT_UPDATE=1 cargo test\n\
                 or: api-drift note --surface {} \"<migration note>\"\n\
                 breaks:",
                surface.name(),
                breaks.len(),
                surface.name()
            );
            for b in &breaks {
                use std::fmt::Write as _;

                let _ = writeln!(
                    msg,
                    "  [{:?}/{:?}] {} — {}",
                    b.severity, b.kind, b.path, b.note
                );
            }
            panic!("{msg}");
        }
    }
}

/// The embedded-ledger test. Every surface in the list is checked against
/// `dir` (see crate docs for the decision tree). Surfaces are `;`-terminated
/// expressions (a `,`-separated list would make `macro_rules!` choke on a
/// nested `enum_surface!(…)` call — `;` is the safe terminator).
///
/// ```ignore
/// ledger! {
///     dir: "api-drift";
///     enum_surface!("backend-events", bro_tui::backend::BackendEvent,
///         [Output, Disconnected]);
/// }
/// ```
#[macro_export]
macro_rules! ledger {
    (dir: $dir:literal; $($surface:expr);* $(;)?) => {
        #[test]
        fn api_drift_ledger_check() {
            $(
                $crate::check_one(
                    std::path::Path::new($dir),
                    &$surface,
                    std::env::var("API_DRIFT_UPDATE").is_ok_and(|v| v == "1"),
                );
            )*
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use api_drift::surface::EnumSurface;
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "api-drift-ledger-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn surf(name: &'static str, variants: &[(&str, &str)]) -> EnumSurface {
        EnumSurface::new(name, "m::E", variants)
    }

    #[test]
    fn first_record_initializes() {
        let d = temp_dir("init");
        let s = surf("e", &[("A", ""), ("B", "")]);
        let out = check(&d, &s, false);
        assert!(out.is_err()); // no committed snapshot + no update → error
        let out = check(&d, &s, true).unwrap();
        assert!(matches!(out, CheckOutcome::Initialized));
        // Now clean.
        assert!(matches!(check(&d, &s, false).unwrap(), CheckOutcome::Clean));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn unrecorded_drift_is_detected() {
        let d = temp_dir("drift");
        let a = surf("e", &[("A", "")]);
        check(&d, &a, true).unwrap();
        // Surface gains a variant.
        let b = surf("e", &[("A", ""), ("B", "")]);
        let out = check(&d, &b, false).unwrap();
        match out {
            CheckOutcome::Unrecorded { breaks } => {
                assert_eq!(breaks.len(), 1);
                assert_eq!(breaks[0].path, "m::E::B");
            }
            other => panic!("expected Unrecorded, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn recorded_hash_passes_and_corroborates() {
        let d = temp_dir("corrob");
        let a = surf("e", &[("A", "")]);
        check(&d, &a, true).unwrap();
        let b = surf("e", &[("A", ""), ("B", "")]);
        // Simulate the consumer-side "upstream branch ledger records the new
        // hash but our committed snapshot file still points at the old":
        // append a ledger entry for B's hash without rewriting the snapshot.
        let to = api_drift::render_snapshot_file("e", &b.snapshot()).content_hash;
        append_entry(
            &d,
            &LedgerEntry {
                ts: 1,
                surface: "e".to_owned(),
                from: String::new(),
                to: to.clone(),
                version: "m::E".to_owned(),
                breaks: vec![],
                note: "added B".to_owned(),
                author: "tester".to_owned(),
                reference: "T-1".to_owned(),
            },
        )
        .unwrap();
        assert!(matches!(
            check(&d, &b, false).unwrap(),
            CheckOutcome::Recorded
        ));
        assert!(is_recorded(&d, "e", &to).unwrap());
        assert_eq!(corroboration(&d, "e", &to).unwrap(), 1);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn corrupted_ledger_line_fails_closed() {
        let d = temp_dir("badline");
        std::fs::write(ledger_path(&d), "{not json}\n").unwrap();
        assert!(matches!(read_entries(&d), Err(LedgerError::BadLine(1, _))));
        let _ = std::fs::remove_dir_all(&d);
    }
}
