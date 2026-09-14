# APIDRIFT-2 — rustdoc-JSON producer

| Field | Value |
|-------|--------|
| **Status** | done |
| **Milestone** | M1 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

Core needs real input: arniko's public surface as `ApiSnapshot`.

## Goal

`cargo rustdoc -- --output-format json` → `ApiSnapshot` producer spec + prototype.

## Scope

- in: producer design, arniko 0.2.98→0.2.99 demo diff
- out: CLI wiring (M2)

## Acceptance

- [x] Producer spec written (`src/producer.rs` docs + limitations)
- [x] Demo diff on arniko versions runs (`tests/arniko_demo.rs`, synthetic rustdoc docs)
- [x] `cargo test` green (core 15+1; `--features producer` 19+1+1)

## Notes

Chosen: rustdoc JSON (nightly `-Z unstable-options`), validated live against
format v61 on this crate (55 public items probed). Alternatives considered:
`cargo public-api` (extra tool install, same data), tree-sitter pass
(heavier, deferred to M2 if rustdoc proves insufficient).

## Resolution

`src/producer.rs` behind `producer` feature (serde_json-gated; core stays
zero-dep): public + in-crate (`crate_id == 0`) + known `inner` kinds only
(fn/struct/enum/trait/field/variant/const), sig = compact `inner` JSON,
unknown shapes skipped not errored. 4 unit tests + `tests/arniko_demo.rs`
end-to-end (widened sig, rename fold, removal, addition). Clippy `-D warnings`
clean both with and without the feature.

