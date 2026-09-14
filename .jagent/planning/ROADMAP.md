# Roadmap — api-drift

Living plan. Dejavue holds *why*; this file holds *sequence*.
Design proposal: `docs/DESIGN-embedded-ledger.md` (proposed, 2026-09-14).

## North star

Upstream API moves once; every downstream fixes itself (or gets told exactly how).

## Phases

| Phase | Name | Goal | Exit criteria |
|-------|------|------|----------------|
| **M0** | Core pipeline | snapshot/diff/classify/suggest types + tests | `cargo test` green, zero deps ✅ |
| **M1** | Producer + hardening | rustdoc-JSON producer, review prereqs | producer + APIDRIFT-4 green ✅ |
| **M2** | Embedded ledger | ledger test in upstreams, consumer check, CLI, gate | bro-acp pair proves it pre-merge |

## Milestone detail

### M0 — Core pipeline

- [x] snapshot/diff/classify/suggest core — APIDRIFT-1
- [x] M0 verification green — APIDRIFT-1

See `milestones/M0-core-pipeline.md`.

### M1 — Producer + hardening

- [x] rustdoc-JSON producer + arniko demo — APIDRIFT-2
- [x] core correctness (last-wins, item_kind, kind-checked rename, Breaking
      sig-change + allowlist, Variant/Field warnings) — APIDRIFT-4

### M2 — Embedded ledger (order: 5 → 6 → 8+9 → 7 → 10 → 11)

- [x] action enum + attrs + serde + snapshot v1 — APIDRIFT-5
- [ ] Surface + ledger crate + dogfood — APIDRIFT-6
- [ ] protocol surface + bro-tui embed — APIDRIFT-8 ∥ consumer + bro-desktop — APIDRIFT-9
- [ ] rust-public surface + arniko ledger demo — APIDRIFT-7 (folds in APIDRIFT-2)
- [ ] CLI — APIDRIFT-10 → registry/gate — APIDRIFT-11
- [ ] call-site applier — APIDRIFT-3 (after 5 + 9)

## Non-goals (standing)

- Semantic type-aware diffing (producer's job, not the core's)
- Auto-landing patches without review (suggest, don't push)
- Runtime registration or reflection (test-time + file-based only)

## Version tags (when releasing)

| Tag | Maps to |
|-----|---------|
| v0.1.0 | M0 complete |
