//! `api-drift` CLI — operate on a repo's embedded ledger without writing Rust.
//!
//! Upstream-side subcommands (`check`, `note`) are live; `affected` /
//! `release-hint` arrive with the consumer registry (APIDRIFT-11) and
//! consumer check (APIDRIFT-9). `--json` output is for agents.

use api_drift::parse_snapshot_file;
use api_drift_ledger::{append_entry, read_entries, LedgerEntry};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(
    name = "api-drift",
    version,
    about = "operate on a repo's embedded API-drift ledger"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Integrity check: parse every committed snapshot + ledger line, verify
    /// content hashes. Fail-closed on any corruption.
    Check {
        /// Ledger directory (repo-root `api-drift/`).
        #[arg(long, default_value = "api-drift")]
        dir: PathBuf,
        /// Machine-readable output for agents.
        #[arg(long)]
        json: bool,
    },
    /// Append a migration note for a surface (the producer's decision record).
    /// `to` = the surface's current committed snapshot hash.
    Note {
        /// Ledger directory (repo-root `api-drift/`).
        #[arg(long, default_value = "api-drift")]
        dir: PathBuf,
        /// Surface name, e.g. `acp-contract`. Keys `<surface>.snapshot.json`.
        #[arg(long)]
        surface: String,
        /// The migration note — what changed and what downstream should do.
        #[arg(long)]
        note: String,
        /// Ticket/reference, e.g. `BRO-98`.
        #[arg(long, default_value = "")]
        reference: String,
        /// Who is recording it.
        #[arg(long, default_value = "")]
        author: String,
        /// Machine-readable output for agents.
        #[arg(long)]
        json: bool,
    },
    /// Not yet wired — needs the consumer registry (APIDRIFT-11).
    Affected {
        #[arg(long, default_value = "api-drift")]
        _dir: PathBuf,
    },
    /// Not yet wired — needs the consumer check (APIDRIFT-9).
    ReleaseHint {
        #[arg(long, default_value = "api-drift")]
        _dir: PathBuf,
    },
}
fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("api-drift: {e:#}");
            1
        }
    };
    std::process::exit(code);
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    match cli.command {
        Command::Check { dir, json } => cmd_check(&dir, json),
        Command::Note {
            dir,
            surface,
            note,
            reference,
            author,
            json,
        } => cmd_note(&dir, &surface, &note, &reference, &author, json),
        Command::Affected { .. } => {
            anyhow::bail!("`affected` needs the consumer registry (APIDRIFT-11) — not wired yet");
        }
        Command::ReleaseHint { .. } => {
            anyhow::bail!("`release-hint` needs the consumer check (APIDRIFT-9) — not wired yet");
        }
    }
}

#[derive(serde::Serialize)]
struct SurfaceReport {
    surface: String,
    version: String,
    content_hash: String,
    ledger_entries: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn cmd_check(dir: &Path, json: bool) -> anyhow::Result<i32> {
    let entries = read_entries(dir).map_err(|e| anyhow::anyhow!("ledger read failed: {e}"))?;

    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("cannot read ledger dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".snapshot.json"))
        })
        .collect();
    files.sort();

    let mut reports = Vec::with_capacity(files.len());
    let mut failures = 0usize;
    for file in &files {
        let surface = file
            .file_name()
            .and_then(|n| n.to_str())
            .map_or("?", |n| n.trim_end_matches(".snapshot.json"))
            .to_owned();
        let text = std::fs::read_to_string(file)?;
        match parse_snapshot_file(&text) {
            Ok(sf) => {
                let n = entries.iter().filter(|e| e.surface == sf.surface).count();
                reports.push(SurfaceReport {
                    surface: sf.surface.clone(),
                    version: sf.version.clone(),
                    content_hash: sf.content_hash.clone(),
                    ledger_entries: n,
                    error: None,
                });
            }
            Err(e) => {
                failures += 1;
                reports.push(SurfaceReport {
                    surface,
                    version: "-".to_owned(),
                    content_hash: "-".to_owned(),
                    ledger_entries: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
    } else {
        for r in &reports {
            match &r.error {
                None => println!(
                    "{}  {}  {}  ({} ledger entries)",
                    r.surface, r.version, r.content_hash, r.ledger_entries
                ),
                Some(e) => println!("{}  ERROR: {e}", r.surface),
            }
        }
    }

    Ok(i32::from(failures > 0))
}

fn cmd_note(
    dir: &Path,
    surface: &str,
    note: &str,
    reference: &str,
    author: &str,
    json: bool,
) -> anyhow::Result<i32> {
    let snap_path = dir.join(format!("{surface}.snapshot.json"));
    let text = std::fs::read_to_string(&snap_path).map_err(|_| {
        anyhow::anyhow!(
            "no committed snapshot for `{surface}` — run `API_DRIFT_UPDATE=1 cargo test` first"
        )
    })?;
    let sf = parse_snapshot_file(&text)?;
    let entries = read_entries(dir)?;

    // `from` chains off the previous entry's `to`; if that entry was the
    // auto-recorded one (empty note), carry its breaks so the note keeps the
    // diff detail.
    let prev = entries.iter().rev().find(|e| e.surface == surface);
    let from = prev.map(|e| e.to.clone()).unwrap_or_default();
    let breaks = prev
        .filter(|e| e.note.trim().is_empty())
        .map(|e| e.breaks.clone())
        .unwrap_or_default();

    let entry = LedgerEntry {
        ts: unix_secs(),
        surface: surface.to_owned(),
        from,
        to: sf.content_hash.clone(),
        version: sf.version.clone(),
        breaks,
        note: note.to_owned(),
        author: author.to_owned(),
        reference: reference.to_owned(),
    };
    append_entry(dir, &entry)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "surface": entry.surface,
                "from": entry.from,
                "to": entry.to,
                "version": entry.version,
                "breaks": entry.breaks.len(),
                "note": entry.note,
                "reference": entry.reference,
            }))?,
        );
    } else {
        println!(
            "recorded {}: {} -> {} ({})",
            entry.surface, entry.from, entry.to, entry.version
        );
    }
    Ok(0)
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
