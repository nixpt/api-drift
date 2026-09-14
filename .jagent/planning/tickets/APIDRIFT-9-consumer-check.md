# APIDRIFT-9 — consumer side: api-drift.toml pins, check!, syn usage scan

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

Ledger records upstream changes; nothing intersects them with what a
downstream actually uses, scoped to call sites.

## Goal

`api-drift.toml` upstream pins, `check!` test, `syn` usage scan of the
consumer's own source (use paths, method calls, enum patterns) →
call-site-scoped suggestions. Embed in bro-desktop against bro-tui's ledger.

## Scope

- in: pins, check macro, usage scan, bro-desktop embed
- out: CLI, registry

## Acceptance

- [ ] `check!` reports breaks ∩ used items with file:line call sites
- [ ] bro-desktop shows the BRO-98 break (src/agent.rs:102) pre-merge
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Design doc § "The consumer check, concretely". Depends on APIDRIFT-6.
Pairs with APIDRIFT-8.
