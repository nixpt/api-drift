# APIDRIFT-8 — protocol surface (ACP/MCP) + embed in bro-tui for BRO-98

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

bro's real contract is ACP methods + JSON schemas + `BackendEvent` variants,
not Rust `pub` items. BRO-98 rewrites `BackendEvent` and adds `_bro/*`
extension methods — bro-desktop matches exhaustively today and will break.

## Goal

`protocol` surface: ACP/MCP methods with request/response schema hashes as
sig; `_bro/*` extensions first. Embed in bro-tui (coordinate with bro-cli
BRO-98's horse); the break must be inferred before merge.

## Scope

- in: protocol surface, bro-tui embed, BRO-98 coordination
- out: CLI, registry

## Acceptance

- [ ] protocol surface snapshots bro-tui's ACP contract
- [ ] BRO-98 diff shows the BackendEvent break pre-merge
- [ ] `cargo test` green (or the project's equivalent)

## Notes

First real customer per the design doc. Depends on APIDRIFT-6. Runs in
parallel with APIDRIFT-9 (the bro-desktop pair is the demo that proves it).
