# APIDRIFT-15 — x-ray → nexus: `xray inspect --json` as the first cross-repo consumer pair

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-16 |
| **Updated** | 2026-09-16 |

## Problem

`xray inspect --json` carries `meta.schema_version` (x-ray `crates/x-ray-tui/src/inspect.rs:78,240`,
XRAY-36) and nexus gates on it by hand (`crates/nexus-ui/src/xray_inspect.rs`:
`SUPPORTED_SCHEMA_MAJOR`, `UnsupportedSchema`). Two producer changes are queued (XRAY-37 health
snapshot in the JSON, XRAY-38 CI smoke for the guest CLI probes) and nothing classifies them
for the consumer. This is APIDRIFT-9's shape on a pair that is *not* in flux — the M2 unblock.

## Goal

x-ray commits an `api-drift/` ledger for an `inspect-json` `ProtocolSurface`; nexus pins it in
`api-drift.toml` and runs `check!` in `cargo test`; the hand-rolled major gate stays as the
runtime guard but its *value* is derived from the pinned snapshot, not typed twice.

## Scope

- in: producer sig for a JSON report (decide: derive `schemars::JsonSchema` on x-ray's report
  structs and use `schema_hash`, or hash a canonical fixture — record the choice in
  `docs/CONSUMERS.md`); `ledger!` test in x-ray; `check!` in nexus with a Rust usage scan of
  `xray_inspect.rs` field accesses; one recorded ledger note when XRAY-37 lands.
- out: the TUI health drill itself; polydex.

## Acceptance

- [ ] `cargo test` in x-ray fails with the classified break + recording command when a
      top-level `inspect --json` field is added/renamed without a note.
- [ ] `cargo test` in nexus fails when the pinned x-ray snapshot hash moves past a
      Breaking entry that touches a field `xray_inspect.rs` reads; passes on Compatible ones.
- [ ] XRAY-37's field addition is recorded as the first real ledger entry (Compatible).
