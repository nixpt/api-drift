# APIDRIFT-7 — rust-public surface via rustdoc JSON

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

Current `producer` module is standalone; not wired as a `Surface`.

## Goal

`rust-public` surface (feature-gated) over rustdoc JSON (nightly is on the
box); arniko 0.2.98→0.2.99 demo becomes a ledger test. APIDRIFT-2 folds in
here — the existing producer is the starting point, not a duplicate.

## Scope

- in: Surface impl for rust-public, arniko ledger demo
- out: protocol/consumer surfaces

## Acceptance

- [ ] rust-public surface produces snapshots from rustdoc JSON
- [ ] arniko demo runs as a ledger test
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Depends on APIDRIFT-6. Nightly toolchain at `/build/rustup`.
