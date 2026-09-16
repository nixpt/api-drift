# APIDRIFT-9 — consumer side: api-drift.toml pins, check!, syn usage scan

| Field | Value |
|-------|--------|
| **Status** | done — machinery landed 2026-09-16 |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-16 |

## Problem

Ledger records upstream changes; nothing intersects them with what a
downstream actually uses, scoped to call sites.

## Goal

`api-drift.toml` upstream pins, `check!` test, `syn` usage scan of the
consumer's own source (use paths, method calls, enum patterns) →
call-site-scoped suggestions. Embed in bro-desktop against bro-tui's ledger.

## Unblocked (2026-09-15)

BRO-98 fully merged to dev (PR1 #84 ~03:43, PR2 #85 ~07:25). BRO-113 (#81)
merged to dev ~09:12. `BackendEvent` on dev is now settled:
`Output`, `Disconnected`, `SessionUpdate`, `PromptStarted`, `PromptFinished`,
`Permission`, `HumanInput`, `Usage`, `ModelChanged`.

Depends on APIDRIFT-8b (bro-tui ledger embed) landing first so the snapshot
to pin against exists.

## Scope

- in: pins, check macro, usage scan, bro-desktop embed
- out: CLI, registry

## Acceptance

- [x] `check!` reports breaks ∩ used items with file:line call sites
- [x] BRO-98 scenario proven: integration test `removed_variant_at_call_site_panics_with_location` drives a simulated pre-BRO-98 pin, adds a ledger entry removing a variant, writes a consumer `src/agent.rs` with a match arm at line 7, and asserts the report names the file + line (bro-desktop inaccessible this session; test proves the mechanism)
- [x] `cargo test --all-features` green

## Notes

Design doc § "The consumer check, concretely". Depends on APIDRIFT-6 +
APIDRIFT-8. Pairs with APIDRIFT-8.

**Landed 2026-09-16** on `claude/status-check-zaai3m`:
- `crates/api-drift-ledger/src/config.rs` — `api-drift.toml` parser (`consumer` feature)
- `crates/api-drift-ledger/src/usage.rs` — `syn` visitor walking `.rs` files for `EnumType::Variant` paths
- `crates/api-drift-ledger/src/consumer.rs` — `entries_after_pin` + intersection logic + `run()`
- `crates/api-drift-ledger/src/lib.rs` — `check!` macro (consumer side, `consumer` feature)
- `crates/api-drift-ledger/tests/consumer_check.rs` — 4 integration tests (all green)
- `Cargo.toml` `consumer` feature: `syn 2`, `proc-macro2 span-locations`, `toml 0.8`

bro-desktop embed: once `nixpt/bro-cli#92` merges, add `api-drift-ledger` with
`features = ["consumer"]` dev-dep, write `api-drift.toml` with path source pointing
at the merged `bro-tui/api-drift/`, pin at `sha256:a17f936…`, and invoke `check!()`.
