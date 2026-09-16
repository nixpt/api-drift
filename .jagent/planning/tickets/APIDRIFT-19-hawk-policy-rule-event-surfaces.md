# APIDRIFT-19 — hawk: `PolicyRule` kinds + `SecurityEvent` sink as surfaces; `policies/*.yaml` as a YAML consumer

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | S |
| **Owner** | unassigned |
| **Created** | 2026-09-16 |
| **Updated** | 2026-09-16 |

## Problem

`hawk policy lint` is a hand-rolled surface check over the YAML `kind:` strings of
`PolicyRule` (`src/policy.rs:30`); HAWK-9 adds three kinds (`unit_restart_guard`,
`timer_fail_streak`, `zombie_parent`). The `SecurityEvent` JSON sink (`src/lib.rs:86`,
HAWK-3) is consumed by alert-hook scripts with no contract. Both are cheap, and the YAML
consumer is the first non-Rust usage-scan target.

## Goal

`enum_surface!` over `PolicyRule` (sig = the YAML fields per kind), `protocol_surface!` over
the event sink (`schema_hash` via schemars), ledger in hawk; `check!` with a YAML extractor
that reads `kind:` values from `policies/*.yaml` and fails on an unknown kind *before*
`hawk policy lint` would at runtime.

## Scope

- in: two surfaces, ledger, YAML extractor for `check!`; HAWK-9 as the first entry.
- out: the rules themselves (HAWK-9).

## Acceptance

- [ ] A `PolicyRule` variant or a field rename without a note fails hawk tests.
- [ ] A `policies/*.yaml` with a `kind:` not in the pinned snapshot fails `cargo test`.
