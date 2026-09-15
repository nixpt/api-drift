# APIDRIFT-7 — rust-public surface via rustdoc JSON

| Field | Value |
|-------|--------|
| **Status** | done |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-15 |

## Problem

Current `producer` module is standalone; not wired as a `Surface`.

## Goal

`rust-public` surface (feature-gated) over rustdoc JSON (nightly is on the
box); arniko 0.2.98→0.2.99 demo becomes a ledger test. APIDRIFT-2 folds in
here — the existing producer is the starting point, not a duplicate.

## Acceptance

- [x] rust-public surface produces snapshots from rustdoc JSON
- [x] arniko demo runs as a ledger test
- [x] `cargo test` green: 45 core / 53 all-features; clippy + rustdoc `-D warnings` + `cargo fmt --check` clean

## Notes

Depends on APIDRIFT-6. The demo uses synthetic rustdoc JSON (not a real
`cargo +nightly rustdoc` run) so it stays deterministic + self-contained;
real rustdoc output drifts per toolchain and needs nightly + a full build.

## Resolution

- `surface.rs`: `RustPublicSurface` (behind `producer` feature) — parses
  rustdoc JSON **eagerly** at construction (`from_str`/`from_json_file`),
  `name()` returns the fixed `"rust-public"`, `snapshot()` clones the parsed
  `ApiSnapshot`. Reuses APIDRIFT-2's `snapshot_from_rustdoc_str/json`
  unchanged.
- `lib.rs`: re-export `RustPublicSurface` (feature-gated).
- `tests/rust_public_demo.rs`: `ledger!` over a `RustPublicSurface` built
  from synthetic arniko 0.2.99 rustdoc JSON (`Alert::new` fn + `Badge`
  struct); committed `api-drift/rust-public.snapshot.json` + ledger entry.

