# APIDRIFT-1 — M0 core pipeline scaffold

| Field | Value |
|-------|--------|
| **Status** | done |
| **Milestone** | M0 |
| **Size** | S |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

Shared crates (arniko) move APIs; downstream repos break and need hand edits.
No mechanical layer exists to say what changed, how bad, what fixes it.

## Goal

I/O-free core: `ApiSnapshot` → `ApiDiff` → `ClassifiedBreak` → `Suggestion`,
pure std, zero deps, tested.

## Scope

- in: snapshot/diff/classify/suggest modules + unit tests + fleet template dots
- out: producers (rustdoc-JSON), CLI, appliers (APIDRIFT-2/3)

## Acceptance

- [x] `cargo test` green (15 tests + 1 doctest, 16/16)
- [x] `cargo clippy --all-targets -- -D warnings` green

## Notes

Scaffolded from `nixpt-common/templates/rust` + `pid-event-policy` conventions.
