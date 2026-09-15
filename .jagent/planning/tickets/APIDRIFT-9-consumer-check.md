# APIDRIFT-9 — consumer side: api-drift.toml pins, check!, syn usage scan

| Field | Value |
|-------|--------|
| **Status** | ready — BRO-98 (PR1 #84 + PR2 #85) and BRO-113 (#81) all merged to dev 2026-09-15 |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-15 |

## Problem

Ledger records upstream changes; nothing intersects them with what a
downstream actually uses, scoped to call sites.

## Goal

`api-drift.toml` upstream pins, `check!` test, `syn` usage scan of the
consumer's own source (use paths, method calls, enum patterns) →
call-site-scoped suggestions. Embed in bro-desktop against bro-tui's ledger.

## Unblocked (2026-09-15)

BRO-98 fully merged to dev (PR1 #84 ~03:43, PR2 #85 ~07:25). BRO-113 (#81)
merged to dev ~09:12. `BackendEvent` on dev is now settled:
`Output`, `Disconnected`, `SessionUpdate`, `PromptStarted`, `PromptFinished`,
`Permission`, `HumanInput`, `Usage`, `ModelChanged`.

Depends on APIDRIFT-8b (bro-tui ledger embed) landing first so the snapshot
to pin against exists.

## Scope

- in: pins, check macro, usage scan, bro-desktop embed
- out: CLI, registry

## Acceptance

- [ ] `check!` reports breaks ∩ used items with file:line call sites
- [ ] bro-desktop shows the BRO-98 break (src/agent.rs:102) pre-merge
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Design doc § "The consumer check, concretely". Depends on APIDRIFT-6 +
APIDRIFT-8. Pairs with APIDRIFT-8.
