# Handoff

Updated: 2026-09-15

## Summary

api-drift (nixpt/api-drift, private) — embedded contract ledger for API drift.
This stretch (2026-09-14→15) shipped everything that is unblocked on its own;
the rest of the M2 board is gated on `bro-cli` settling, on purpose.

Landed this session (pushed `99bbdfe` tip):
- **M0 core** — snapshot/diff/classify/suggest (pure std, zero deps, unsafe deny).
- **APIDRIFT-4** review hardening — old-side last-wins, `item_kind` +
  kind-checked rename (`review-rename` on weak signal), `SignatureChanged` →
  Breaking + widening allowlist, Added Variant/Field → Warning.
- **APIDRIFT-5** `SuggestionAction` enum + `Item.attrs` + `serde` feature +
  snapshot file format v1 (sha256 content hash, fail-closed).
- **APIDRIFT-6** workspace conversion → `crates/api-drift` (core) +
  `crates/api-drift-ledger` (`Surface`, `EnumSurface`, `enum_surface!`,
  `CatalogSurface`, `ProtocolSurface`, `RustPublicSurface`, `schema_hash`,
  `LedgerEntry`/`record`/`check`/`corroboration`, `ledger!` test macro);
  dogfood ledger over `ItemKind` committed in `api-drift/`.
- **APIDRIFT-7** `RustPublicSurface` (rustdoc JSON) + arniko ledger demo.
- **APIDRIFT-8a** protocol surface + schema-hash sig.
- **APIDRIFT-10** `api-drift` CLI: `check` + `note` (`--json`); affected/
  release-hint stubbed.
- **APIDRIFT-12** borrows — antibody notes + `corroboration`, `autoimmune_guard`
  + `Suggestion.confidence`.
- **APIDRIFT-13/14** design surveys + `ItemKind::Row` (`CatalogSurface`) for
  checkstand's product catalog.

Health: 48 core lib tests / 53 `--workspace --all-features`; clippy +
rustdoc `-D warnings` + stable `cargo fmt --check` all clean. (Builds run on
`/tmp/api-drift-target` — `/build` is full.)

## Next Steps (gated on bro-cli settling — do NOT edit bro-cli until then)

1. **APIDRIFT-8b** — embed the ledger in bro-tui (coordinate with the BRO-98
   horse): `ledger!` with `protocol_surface!` + `enum_surface!` over the ACP
   contract + `BackendEvent`.
2. **APIDRIFT-9** — consumer side: `api-drift.toml` pins, `check!`, `syn`
   usage scan, embed in bro-desktop (`src/agent.rs:102` exhaustive match).
3. **APIDRIFT-10 remainder** — `affected` / `release-hint` (needs 9/11).
4. **APIDRIFT-11** — `consumers.toml` registry + foreman merge-gate hook
   (relatedness ranking per APIDRIFT-12).
5. **APIDRIFT-3** — call-site applier (after 5 + 9).
6. **APIDRIFT-12 remainder** — structural-similarity rename metric (evaluate
   vs edit-distance ≤ 8) + signed `did:key` ledger notes (deferred).

## Boot Instructions

Read `.dejavue/handoff.md`, `.dejavue/state.md`, `.jagent/planning/STATE.md`
+ `TASKS.md`, and `docs/DESIGN-embedded-ledger.md` before making changes.
The board lives in `.jagent/planning/` (`APIDRIFT-NN` tickets, one per work
item). Sync channel: `scripts/api-drift-sync` → `.jagent/sync/api-drift.jsonl`
(gitignored). Reach out to the foreman before resuming anything gated on
bro-cli.

