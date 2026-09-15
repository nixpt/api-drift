# APIDRIFT-6 — Surface trait + enum_surface! + api-drift-ledger crate

| Field | Value |
|-------|--------|
| **Status** | done |
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

## Acceptance

- [x] `ledger!` passes with no diff, fails with unrecorded diff + note cmd
- [x] `API_DRIFT_UPDATE=1` records locally; CI runs without it
- [x] Dogfood ledger committed on api-drift itself (`api-drift/item-kind.*`)
- [x] `cargo test` green: 35 core / 41 all-features / 0 errors; clippy `-D warnings` + rustdoc `-D warnings` + `cargo fmt --check` clean

## Notes

Design doc § "The ledger test, concretely". Depends on APIDRIFT-5. Also folds
in APIDRIFT-12's first two borrows (note-as-antibody + `corroboration`).

## Resolution

- **Workspace conversion**: repo root is now a virtual `[workspace]` with
  members `crates/api-drift` (core, moved from `src/`) and
  `crates/api-drift-ledger`; lints/deps/package metadata inherit from
  `[workspace.*]`. Upstream release metadata (rust-version 1.74, keywords,
  `assets/` exclude) preserved.
- **`src/surface.rs`** (core, pure std): `Surface` trait, `EnumSurface`
  (variant list → `ItemKind::Variant` items), `enum_surface!` macro
  (comma/bracket form so it nests inside `ledger!`).
- **`api-drift-ledger`**: `LedgerEntry` (with `note`, `author`, `ref`,
  `breaks`, `from`/`to` hashes), `read_entries`/`append_entry` (append-only
  JSONL, fail-closed on bad lines), `is_recorded`, `corroboration`,
  `record`, `check` (Clean/Recorded/Unrecorded/Updated/Initialized),
  `check_all`/`check_one`, and the `ledger!` test macro (`;`-terminated
  surface list).
- **Dogfood**: `crates/api-drift/tests/dogfood.rs` embeds a `ledger!` over
  `snapshot::ItemKind`, with committed `api-drift/item-kind.snapshot.json`
  + `api-drift/ledger.jsonl`.
- **Tests**: 29 core lib + 1 dogfood + 4 ledger unit tests
  (init/drift-drill/recorded-hash+bad-line fail-closed).

