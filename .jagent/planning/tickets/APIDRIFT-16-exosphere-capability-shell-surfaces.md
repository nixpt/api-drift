# APIDRIFT-16 — exosphere: `CapabilityType` + ExoShell protocol + `CapsuleManifest` surfaces; nexus `Request` mirror as consumer

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | L |
| **Owner** | unassigned |
| **Created** | 2026-09-16 |
| **Updated** | 2026-09-16 |

## Problem

Three exosphere contracts have downstream hand copies and pending changes:
- `CapabilityType` (`crates/core/base/common/src/capability.rs:6`) — EXO-214 adds a `Tty`
  family; every exhaustive `match` downstream (spawner, policy, nexus manifest tooling) breaks.
- `ShellRequest`/`ShellResponse` (`crates/exo/shell-client/src/protocol.rs:135,192`) — nexus
  keeps a serde-tagged **hand copy** as `nexus_session::exosphere::Request`
  (`crates/nexus-session/src/exosphere.rs:73`); EXO-215 adds fields/ops.
- `CapsuleManifest` is **defined twice** in exosphere (`daemon/src/enforcement.rs:575`,
  `integration/src/api.rs:56`) and consumed by nexus `packaging/exosphere/Capsule.toml` and
  nakshatra's capsule staging (NAK-14b).

## Goal

`enum_surface!` over `CapabilityType`, `protocol_surface!` over the shell wire (method names +
`schema_hash` of request/response), `catalog_surface!` over manifest keys — one ledger in
exosphere; nexus pins `shell-wire` and the ledger test fails when `Request` and `ShellRequest`
disagree. The duplicate manifest structs are held to one surface so they cannot diverge.

## Scope

- in: the three surfaces + ledger in exosphere; `check!` in nexus for `shell-wire`; EXO-214
  and EXO-215 recorded as the first entries (Breaking for the enum variant, with the
  `add-match-arm` suggestion; Compatible for optional protocol fields).
- out: unifying the two `CapsuleManifest` definitions (file separately if the surface proves
  they already differ); exosphere's ~15 absolute path-deps problem (`.jagent/TODO.md`).

## Acceptance

- [ ] Adding a `CapabilityType` variant without a ledger note fails exosphere's tests with
      `Added Variant … Breaking` and the recording command.
- [ ] nexus `cargo test` fails when its `Request` mirror lacks a method present in the pinned
      `shell-wire` snapshot (and names it).
- [ ] The two `CapsuleManifest` key sets are proven equal by the catalog surface, or the
      difference is recorded as a finding.
