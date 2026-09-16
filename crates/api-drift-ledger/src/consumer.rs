//! Consumer-side check (APIDRIFT-9): reads `api-drift.toml`, follows the
//! upstream ledger from `pin` to head, intersects classified breaks with a
//! `syn` usage scan of the consumer's own `src/`, and panics with a
//! call-site-scoped report when any break affects a used item.
//!
//! Entry point: [`run`] — called by the [`check!`](crate::check) macro.

use crate::config::{read_config, UpstreamSource};
use crate::usage::{scan, UsageSite};
use crate::{read_entries, LedgerEntry};
use api_drift::classify::ClassifiedBreak;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// A classified break together with the consumer call sites where it's used.
#[derive(Debug)]
pub struct BreakWithSites {
    pub brk: ClassifiedBreak,
    pub sites: Vec<UsageSite>,
    /// Migration note from the upstream ledger entry.
    pub migration_note: String,
    /// Upstream ticket reference.
    pub reference: String,
}

/// Run the consumer check from `manifest_dir` (i.e. `env!("CARGO_MANIFEST_DIR")`).
///
/// Reads `<manifest_dir>/api-drift.toml`, resolves each `[[upstream]]` pin,
/// scans `<manifest_dir>/src/` with `syn`, intersects the ledger breaks with
/// used items, and panics with a formatted report when any break hits a real
/// call site.
///
/// Passes silently when the consumer is up to date with its pin.
pub fn run(manifest_dir: &str) {
    let config_path = Path::new(manifest_dir).join("api-drift.toml");
    let config =
        read_config(&config_path).unwrap_or_else(|e| panic!("api-drift consumer check: {e}"));

    let src_dir = Path::new(manifest_dir).join("src");
    let mut report = String::new();
    let mut has_break = false;

    for upstream in &config.upstream {
        let ledger_dir = resolve_source(manifest_dir, &upstream.source);
        let entries = read_entries(&ledger_dir).unwrap_or_else(|e| {
            panic!(
                "api-drift consumer: reading ledger for `{}`: {e}",
                upstream.name
            )
        });

        let new_entries = entries_after_pin(&entries, &upstream.surface, &upstream.pin);
        if new_entries.is_empty() {
            continue;
        }

        // Collect (break, migration_note, ref) from all new entries.
        let annotated: Vec<(ClassifiedBreak, &str, &str)> = new_entries
            .iter()
            .flat_map(|e| {
                e.breaks
                    .iter()
                    .map(move |b| (b.clone(), e.note.as_str(), e.reference.as_str()))
            })
            .collect();

        if annotated.is_empty() {
            continue;
        }

        // Scan consumer source for usages of the broken item paths.
        let item_paths: Vec<&str> = annotated.iter().map(|(b, _, _)| b.path.as_str()).collect();
        let sites = if src_dir.exists() {
            scan(&src_dir, &item_paths)
        } else {
            Vec::new()
        };

        // Intersect: keep only breaks that have at least one call site.
        let breaks_with_sites: Vec<BreakWithSites> = annotated
            .into_iter()
            .filter_map(|(brk, note, reference)| {
                let these: Vec<_> = sites
                    .iter()
                    .filter(|s| s.item_path == brk.path)
                    .cloned()
                    .collect();
                if these.is_empty() {
                    None
                } else {
                    Some(BreakWithSites {
                        brk,
                        sites: these,
                        migration_note: note.to_owned(),
                        reference: reference.to_owned(),
                    })
                }
            })
            .collect();

        if breaks_with_sites.is_empty() {
            continue;
        }

        has_break = true;
        let head = new_entries.last().map_or("", |e| e.to.as_str());
        let refs: Vec<&str> = new_entries
            .iter()
            .filter_map(|e| {
                if e.reference.is_empty() {
                    None
                } else {
                    Some(e.reference.as_str())
                }
            })
            .collect();
        let _ = writeln!(
            report,
            "\n{}/{}  pin {}…  → head {}…  ({} ledger entry/entries{})",
            upstream.name,
            upstream.surface,
            upstream.pin.chars().take(16).collect::<String>(),
            head.chars().take(16).collect::<String>(),
            new_entries.len(),
            if refs.is_empty() {
                String::new()
            } else {
                format!(", ref {}", refs.join(", "))
            },
        );

        for bws in &breaks_with_sites {
            let _ = writeln!(
                report,
                "  {:?}/{:?}  {}",
                bws.brk.severity, bws.brk.kind, bws.brk.path,
            );
            for site in &bws.sites {
                let _ = writeln!(report, "    used: {}:{}", site.file, site.line);
            }
            if !bws.migration_note.is_empty() {
                let _ = writeln!(report, "    note: {}", bws.migration_note);
            }
        }
    }

    assert!(
        !has_break,
        "api-drift consumer check: breaks affecting used items:{report}"
    );
}

fn resolve_source(manifest_dir: &str, source: &UpstreamSource) -> PathBuf {
    match source {
        UpstreamSource::Path { path } => Path::new(manifest_dir).join(path),
    }
}

/// All ledger entries for `surface` that come after the entry where `to == pin`.
///
/// If `pin` is empty, returns all entries for the surface (full history).
/// If `pin` is not found in the ledger, returns empty (conservative: don't
/// report changes we can't position in the chain).
fn entries_after_pin<'a>(
    entries: &'a [LedgerEntry],
    surface: &str,
    pin: &str,
) -> Vec<&'a LedgerEntry> {
    let surface_entries: Vec<&LedgerEntry> =
        entries.iter().filter(|e| e.surface == surface).collect();

    if pin.is_empty() {
        return surface_entries;
    }

    let pivot = surface_entries.iter().rposition(|e| e.to == *pin);
    match pivot {
        Some(i) => surface_entries[i + 1..].to_vec(),
        None => vec![],
    }
}
