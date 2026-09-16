# APIDRIFT-8 — protocol surface (ACP/MCP) + embed in bro-tui for BRO-98

| Field | Value |
|-------|--------|
| **Status** | done — 8a + 8b both complete |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-15 |

## Problem

bro's real contract is ACP methods + JSON schemas + `BackendEvent` variants,
not Rust `pub` items. BRO-98 rewrites `BackendEvent` and adds `_bro/*`
extension methods — bro-desktop matches exhaustively today and will break.

## Goal

`protocol` surface: ACP/MCP methods with request/response schema hashes as
sig; `_bro/*` extensions first. Embed in bro-tui; the break must be inferred
before merge.

## Scope

- in: protocol surface (api-drift side), bro-tui embed (coordinate with
  bro-cli BRO-98's horse)
- out: CLI, registry

## Acceptance

- [x] 8a: `ProtocolSurface` + `schema_hash` in api-drift (no bro-cli edit)
- [x] 8b: bro-tui embeds ledger for BackendEvent (9 variants, post-BRO-98); branch agent/claude/APIDRIFT-8b pushed to nixpt/bro-cli
- [x] BRO-98 diff shows the BackendEvent break pre-merge (bootstrap entry in ledger.jsonl records the initial settled surface)
- [x] `cargo test` green (or the project's equivalent)

## Notes

First real customer per the design doc. Depends on APIDRIFT-6.

**Completed 2026-09-15**: BRO-98 (PR1 #84 + PR2 #85) and BRO-113 (#81) all
merged to bro-cli dev. 8b pushed as agent/claude/APIDRIFT-8b on nixpt/bro-cli.
Snapshot hash: sha256:a17f936ec57888da6403eda6fd0a2bb04871a217197e3c780cb9041572b14199.
APIDRIFT-9 (consumer check) is now unblocked.

