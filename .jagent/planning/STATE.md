# Planning state — api-drift

**Updated:** 2026-09-14
**Milestone focus:** M1 core hardening done (APIDRIFT-4); M2 embedded-ledger track open
**Branch:** `master` (no commits, no remote — planning said `main`; rename or accept on first commit)

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|-------|
| snapshot/diff/classify/suggest core | **shipped** | pure std, zero deps |
| arniko rustdoc-JSON producer | **shipped** | APIDRIFT-2; `producer` feature, serde_json-gated |
| arniko 0.2.98→0.2.99 demo | **shipped** | tests/arniko_demo.rs (synthetic rustdoc docs) |
| core correctness (review prereqs) | **shipped** | APIDRIFT-4, uncommitted |
| embedded-ledger design | **proposed** | docs/DESIGN-embedded-ledger.md (captain+foreman review) |
| action enum + serde + format v1 | **ready** | APIDRIFT-5 |
| Surface + ledger crate + dogfood | **planned** | APIDRIFT-6 |
| bro-acp pair (protocol + consumer) | **planned** | APIDRIFT-8 ∥ APIDRIFT-9, first customer |

## Active work

_APIDRIFT-4 landed in-tree, uncommitted. Next: APIDRIFT-5 (see TASKS.md for
the full 5→6→8+9→7→10→11 order)._

## Blockers

- Repo has no commits and no remote; git is on `master` while planning says
  `main`. Decide on first commit (rename to `main` or update planning).
- `/build` 100% full — builds run on `/tmp/api-drift-target`; real arniko
  rustdoc snapshots wait on disk budget.

## Metrics

| Metric | Value |
|--------|-------|
| Unit tests (core) | 21 + 1 doctest |
| Unit tests (`--features producer`) | 25 + 1 demo + 1 doctest |
| Test pass rate | 100% |
| Warnings | 0 (clippy `-D warnings` clean ± feature) |

## Next (ranked)

1. APIDRIFT-5 — action enum + attrs + serde + snapshot v1
2. APIDRIFT-6 — Surface + ledger crate + dogfood
3. APIDRIFT-8 + APIDRIFT-9 — bro-tui / bro-desktop pair (the demo)
4. APIDRIFT-7 — rust-public surface + arniko ledger demo
5. APIDRIFT-10 → APIDRIFT-11 — CLI → registry/gate


