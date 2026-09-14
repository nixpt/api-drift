# APIDRIFT-10 — api-drift CLI (check, note, affected, release-hint, --json)

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | S–M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

Agents and merge gates need the ledger without writing Rust.

## Goal

`api-drift` bin: `check`, `note`, `affected`, `release-hint`, `--json` for
agents.

## Scope

- in: CLI crate over the ledger/consumer layers
- out: fleet registry (APIDRIFT-11)

## Acceptance

- [ ] All four subcommands work against a ledgered repo
- [ ] `--json` output is agent-parseable
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Depends on APIDRIFT-9.
