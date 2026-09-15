# APIDRIFT-8 — protocol surface (ACP/MCP) + embed in bro-tui for BRO-98

| Field | Value |
|-------|--------|
| **Status** | in_progress (8a api-drift-side; 8b bro-tui embed gated) |
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
- [ ] 8b: bro-tui embeds it once bro-cli settles (coordinate w/ BRO-98 horse)
- [ ] BRO-98 diff shows the BackendEvent break pre-merge
- [ ] `cargo test` green (or the project's equivalent)

## Notes

First real customer per the design doc. Depends on APIDRIFT-6. **Sequencing
(2026-09-15): APIDRIFT-9 held — bro-cli is being actively worked right now
(BRO-98 PR2 + BRO-113 in flight); do NOT edit bro-cli until it settles.
Parallel-box doctrine: read-only against in-flight `[zorro]`-style repos.**
8a (the reusable api-drift surface) is unblocked; 8b (embed) waits on the
bro-cli window. Uses APIDRIFT-13's schemars-schema-hash sig borrow.

