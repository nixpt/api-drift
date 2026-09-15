# APIDRIFT-9 — consumer side: api-drift.toml pins, check!, syn usage scan

| Field | Value |
|-------|--------|
| **Status** | blocked — bro-cli actively changing (BRO-98 PR2 + BRO-113 in flight) |
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

## Blocked

Held 2026-09-15: bro-cli is being actively worked (BRO-98 PR2 merged,
BRO-113 in flight). Proceed only when bro-cli settles and after APIDRIFT-8's
bro-tui embed lands; coordinate with the BRO-98 horse.

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
