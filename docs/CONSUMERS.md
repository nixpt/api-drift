# Consumers — where api-drift's surfaces already exist in the fleet

Survey date: 2026-09-16. api-drift's M2 (APIDRIFT-8b/9: ledger embed + consumer `check!`)
is blocked on its first customer, bro-cli, being in flux. This note records six
producer → consumer pairs found while working on nexus, x-ray, hawk, exosphere,
nakshatra and wsforge the same day, each of which already carries **hand-rolled drift
discipline or an unrecorded drift incident** — the same signal that made checkstand
(APIDRIFT-13) worth borrowing from. They are candidates to unblock M2 without waiting
for bro-cli. Tickets: APIDRIFT-15 … APIDRIFT-20.

## The map

| # | Producer surface (kind) | Consumer(s) | Evidence it is already drifting / hand-guarded | Ticket |
|---|---|---|---|---|
| 1 | **x-ray** `xray inspect --json` — `meta.schema_version` (`crates/x-ray-tui/src/inspect.rs:78,240`) | **nexus** `crates/nexus-ui/src/xray_inspect.rs` (`SUPPORTED_SCHEMA_MAJOR`, `UnsupportedSchema` error) | A hand-written major-version gate on both sides; XRAY-37 (health snapshot in `--json`) and XRAY-38 (CI smoke for `--help/--version/schema_version`) are pending shape changes with no classifier | **APIDRIFT-15** |
| 2 | **exosphere** `CapabilityType` enum (`crates/core/base/common/src/capability.rs:6`); `ShellRequest`/`ShellResponse` (`crates/exo/shell-client/src/protocol.rs:135,192`); `CapsuleManifest` (**defined twice**: `daemon/src/enforcement.rs:575`, `integration/src/api.rs:56`) | **nexus** `crates/nexus-session/src/exosphere.rs:73` `enum Request` — a serde-tagged **hand copy** of `ShellRequest`; nexus `packaging/exosphere/Capsule.toml`; nakshatra capsule staging | EXO-214 adds a `Tty` variant (exhaustive matches downstream break); EXO-215 adds protocol fields; two manifest structs can already disagree | **APIDRIFT-16** |
| 3 | **nakshatra** `boot.toml` key set (string table in `init/nak-init.c`) and the forwarded `NAK_*` env names | **nexus** staging recipe (`docs/feasibility/exo-tty2-integration.md`, `scripts/exo-tty2-image-stage.sh`); **exosphere** exo-init (reads `NAK_*`) | **Live incident:** the nexus recipe writes `tty`/`rescue_tty`, keys nak-init does not parse — a consumer referencing non-existent producer items, exactly what `check!`'s usage scan exists to fail. NAK-16 adds the keys (first ledger entry) | **APIDRIFT-17** |
| 4 | **nexus** `nexus_sdk::wire::WireKey` (enum, `crates/nexus-sdk/src/wire.rs:40`); broker IPC incl. `ReadTerminalGrid` (NEXUS-057); `nexus_session::keys` (NEXUS-059, rust-public); `guests.d/*.toml` launcher keys (NEXUS-004) | **nexus-gui** (NEXUS-056/057 — the TUI-backend ↔ GUI-frontend pair, same shape as bro-tui ↔ bro-desktop but **not in flux**); out-of-process SDK panels (`NEXUS_PANEL_SOCKET`); exosphere adapter capsule | Three open PRs (#106–#108) each move one of these surfaces this week with no ledger entry | **APIDRIFT-18** |
| 5 | **hawk** `PolicyRule` enum (`src/policy.rs:30`, the YAML `kind:` strings); `SecurityEvent` JSON sink shape (`src/lib.rs:86`, HAWK-3) | `policies/*.yaml` (a **YAML** usage-scan target, not Rust); alert-hook scripts consuming the JSON sink | HAWK-9 adds three rule kinds; `hawk policy lint` is a hand-rolled surface check | **APIDRIFT-19** |
| 6 | **wsforge** `profiles/*.conf` variable set (32 assigned) | `bin/wsforge-{partition,configure,restore}` (36 distinct `$VARS` read) | 36 ≠ 32: already-unmeasured drift between profile and scripts; WSF-2 adds `NEXUS_VT*` keys | **APIDRIFT-20** |

## Why these, in this order

- **#1 first.** Two repos, one JSON contract, both sides already invented a version gate by
  hand, and two pending producer changes are queued. It is the smallest cross-repo pair
  and needs only `ProtocolSurface` + `schema_hash` (shipped in 8a) on the producer and
  `check!` (APIDRIFT-9's remaining half) on the consumer — i.e. it *is* M2, on a stable pair.
- **#3 is the motivating story.** A consumer shipped a recipe against keys that do not
  exist, in a C producer. It proves the surface model is not Rust-only (a `catalog_surface!`
  list kept next to the parser, plus a test that greps the parser for the listed keys so
  list and code cannot disagree).
- **#2 and #4** are the deepest and the most active; they should follow, not lead, because
  EXO-214/215 and NEXUS-056..059 are changing the surfaces this month — good first ledger
  entries once the ledger exists, bad places to be debugging the ledger itself.
- **#5 and #6** are cheap non-Rust-consumer proofs (YAML, shell).

## What is still missing in api-drift for these

- `check!` + usage scan (APIDRIFT-9) — generalised so the scan target can be Rust (`syn`),
  YAML `kind:` values (#5), shell `$VAR` references (#6), or a TOML/markdown key list (#3).
  A tiny per-language "reference extractor" trait is enough; polydex can replace it later.
- A `json_shape` sig helper for #1: `schema_hash` over a schemars schema exists; the
  `inspect --json` producer has no schema type today, so either derive `JsonSchema` on the
  x-ray report structs (XRAY-37 is touching them) or hash a canonical sample. Decide in
  APIDRIFT-15.
- Nothing else: `enum_surface!`, `protocol_surface!`, `catalog_surface!`, the ledger crate
  and `api-drift check --json` cover the producer side of all six today.
