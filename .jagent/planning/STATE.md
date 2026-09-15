# Planning state — api-drift

**Updated:** 2026-09-14
**Milestone focus:** M1 core hardening done (APIDRIFT-4); M2 embedded-ledger track open
**Branch:** `main` @ `nixpt/api-drift` (private, pushed `4d9841f`)

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|-------|
| snapshot/diff/classify/suggest core | **shipped** | pure std, zero deps |
| arniko rustdoc-JSON producer | **shipped** | APIDRIFT-2; `producer` feature, serde_json-gated |
| arniko 0.2.98→0.2.99 demo | **shipped** | tests/arniko_demo.rs (synthetic rustdoc docs) |
| core correctness (review prereqs) | **shipped** | APIDRIFT-4 |
| action enum + attrs + serde + snapshot v1 | **shipped** | APIDRIFT-5 |
| Surface + ledger crate + dogfood | **shipped** | APIDRIFT-6 (workspace: core + api-drift-ledger) |
| embedded-ledger design | **proposed** | docs/DESIGN-embedded-ledger.md (captain+foreman review) |
| bro-acp pair (protocol + consumer) | **next** | APIDRIFT-8 ∥ APIDRIFT-9, first customer |
| Evorium borrows (antibodies/guard/relatedness) | **in progress** | antibodies+corroboration (6); guard (core); relatedness → 11 |
| checkstand borrows + breadboard | **backlog** | APIDRIFT-13: real-server bar + schemars sig → 8/9/6 |
| data/catalog surface | **shipped** | APIDRIFT-14: ItemKind::Row + CatalogSurface |
| protocol surface (api-drift side) | **shipped** | APIDRIFT-8a: ProtocolSurface + schema_hash |
| rust-public surface | **shipped** | APIDRIFT-7: RustPublicSurface + arniko ledger demo |

## Active work

_None right now — APIDRIFT-12's `autoimmune_guard` + `Suggestion.confidence`
landed (evorium borrow #2, core primitive). Remaining unblocked slice:
APIDRIFT-10 CLI `check`/`note` (upstream-side only; `affected`/`release-hint`
need 9/11). APIDRIFT-8b + 9 stay gated on bro-cli settling._

## Blockers

- bro-cli in flux → APIDRIFT-8b + APIDRIFT-9 gated; coordinate with the
  BRO-98 horse.

## Metrics

| Metric | Value |
|--------|-------|
| Unit tests (core lib, no features) | 38 + 1 doctest |
| Unit tests (`--workspace --all-features`) | 48 + 4 ledger + 1 dogfood + 2 demo + 1 doctest |
| Test pass rate | 100% |
| Warnings | 0 (clippy `-D warnings`, rustdoc `-D warnings`, `cargo fmt --check` clean) |

## Next (ranked)

1. APIDRIFT-6 — Surface + ledger crate + dogfood
2. APIDRIFT-8 + APIDRIFT-9 — bro-tui / bro-desktop pair (the demo)
3. APIDRIFT-7 — rust-public surface + arniko ledger demo
4. APIDRIFT-10 → APIDRIFT-11 — CLI → registry/gate


