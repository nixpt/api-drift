# APIDRIFT-6 — Surface trait + enum_surface! + api-drift-ledger crate

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

No `Surface` abstraction; no ledger test macro; no snapshot/ledger file I/O.

## Goal

`trait Surface { fn name(&self) -> &str; fn snapshot(&self) -> ApiSnapshot; }`,
`enum_surface!` for public enums (the `BackendEvent` case), and a sibling
`api-drift-ledger` crate with the `ledger!` test macro + `ledger.jsonl`
history + migration-note prompt. Dogfood on api-drift itself.

## Scope

- in: Surface trait, enum macro, ledger crate, dogfood test
- out: rust-public surface (APIDRIFT-7), protocol surface (APIDRIFT-8)

## Acceptance

- [ ] `ledger!` test passes with no diff, fails with unrecorded diff + note cmd
- [ ] `API_DRIFT_UPDATE=1` records locally; CI runs without it
- [ ] Dogfood ledger committed on api-drift itself
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Design doc § "The ledger test, concretely". Depends on APIDRIFT-5.
