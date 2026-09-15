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
| Evorium borrows (antibodies/guard/relatedness) | **in progress** | APIDRIFT-12: antibodies+corroboration in 6; guard+relatedness in 9/11 |
| checkstand borrows + breadboard | **backlog** | APIDRIFT-13: real-server bar + schemars sig → 8/9/6 |

## Active work

_None — APIDRIFT-6 landed (workspace conversion + api-drift-ledger). Next:
APIDRIFT-8 ∥ APIDRIFT-9 (bro-acp pair)._

## Blockers

_None (upstream scrubbed the stale `/build` blocker; builds run in `/tmp`)._

## Metrics

| Metric | Value |
|--------|-------|
| Unit tests (core lib, no features) | 29 + 1 doctest |
| Unit tests (`--workspace --all-features`) | 34 + 4 ledger + 1 dogfood + 1 demo + 1 doctest |
| Test pass rate | 100% |
| Warnings | 0 (clippy `-D warnings`, rustdoc `-D warnings`, `cargo fmt --check` clean) |

## Next (ranked)

1. APIDRIFT-6 — Surface + ledger crate + dogfood
2. APIDRIFT-8 + APIDRIFT-9 — bro-tui / bro-desktop pair (the demo)
3. APIDRIFT-7 — rust-public surface + arniko ledger demo
4. APIDRIFT-10 → APIDRIFT-11 — CLI → registry/gate


