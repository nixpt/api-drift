# APIDRIFT-10 — api-drift CLI (check, note, affected, release-hint, --json)

| Field | Value |
|-------|--------|
| **Status** | in_progress — `check`+`note` done; `affected`+`release-hint` gated on 9/11 |
| **Milestone** | M2 |
| **Size** | S–M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-15 |

## Problem

Agents and merge gates need the ledger without writing Rust.

## Goal

`api-drift` bin: `check`, `note`, `affected`, `release-hint`, `--json` for
agents.

## Acceptance

- [x] `check` — integrity-verifies committed snapshots + ledger (fail-closed)
- [x] `note` — appends a migration note, chaining `from`/`to` hashes, carrying
      breaks from the auto-recorded empty-note entry
- [ ] `affected` / `release-hint` — gated on APIDRIFT-11 / APIDRIFT-9
      (stubbed: print "not wired yet", exit 2)
- [x] `--json` is agent-parseable
- [x] `cargo test` green (48 core / 53 workspace) + clippy/rustdoc/fmt clean

## Notes

Depends on APIDRIFT-9/11 for the consumer-side subcommands. `check`/`note`
are the upstream-side half, unblocked by bro-cli.

## Resolution

- New member `crates/api-drift-cli` (`bin = "api-drift"`): clap derive CLI.
  `check --dir [--json]` parses every `<name>.snapshot.json` (hash verified)
  + reads `ledger.jsonl` (fail-closed on bad lines), reports per-surface
  `{surface, version, content_hash, ledger_entries}`. `note --dir --surface
  --note [--reference --author --json]` appends a `LedgerEntry` whose `to` =
  the committed snapshot hash, `from` = previous entry's `to`, `breaks`
  carried from an empty-note auto entry. `affected`/`release-hint` bail
  "not wired yet".
- `api-drift`/`api-drift-ledger` now implement `std::error::Error` for
  `SnapshotFileError`/`LedgerError` (anyhow integration).

