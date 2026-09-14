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
| embedded-ledger design | **proposed** | docs/DESIGN-embedded-ledger.md (captain+foreman review) |
| Surface + ledger crate + dogfood | **next** | APIDRIFT-6 |
| bro-acp pair (protocol + consumer) | **planned** | APIDRIFT-8 ∥ APIDRIFT-9, first customer |
| Evorium borrows (antibodies/guard/relatedness) | **backlog** | APIDRIFT-12, woven into 6/9/11 |

## Active work

_None — APIDRIFT-5 landed and pushed. Next: APIDRIFT-6._

## Blockers

_None._

## Metrics

| Metric | Value |
|--------|-------|
| Unit tests (core) | 25 + 1 doctest |
| Unit tests (`--features serde`) | 29 + 1 doctest |
| Unit tests (`--features producer`) | 34 + 1 demo + 1 doctest |
| Unit tests (`--features serde,producer`) | 34 + 1 demo + 1 doctest |
| Test pass rate | 100% |
| Warnings | 0 (clippy `-D warnings` clean on all four combos) |

## Next (ranked)

1. APIDRIFT-6 — Surface + ledger crate + dogfood
2. APIDRIFT-8 + APIDRIFT-9 — bro-tui / bro-desktop pair (the demo)
3. APIDRIFT-7 — rust-public surface + arniko ledger demo
4. APIDRIFT-10 → APIDRIFT-11 — CLI → registry/gate


