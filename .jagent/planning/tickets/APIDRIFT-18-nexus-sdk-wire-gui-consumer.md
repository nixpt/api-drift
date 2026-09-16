# APIDRIFT-18 — nexus: `WireKey` / broker IPC (`ReadTerminalGrid`) / `nexus_session::keys` / `guests.d` surfaces; nexus-gui as the consumer (the bro-tui ↔ bro-desktop shape, not in flux)

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-16 |
| **Updated** | 2026-09-16 |

## Problem

APIDRIFT-8b/9 are blocked because bro-cli's backend/frontend pair is mid-refactor. nexus has
the same pair — TUI core producing a wire contract, a new GUI frontend consuming it — and it
is *converging*, not churning: NEXUS-056 (scaffold), NEXUS-057 (`ReadTerminalGrid`
protocol extension), NEXUS-059 (toolkit-free `nexus_session::keys`) are three open PRs
(#106–#108) that each move a surface this week, with no ledger entry. Surfaces:
`nexus_sdk::wire::WireKey` (`crates/nexus-sdk/src/wire.rs:40`, enum), the broker IPC
(protocol), `nexus_session::keys` (rust-public), and `guests.d/*.toml` launcher keys
(NEXUS-004, catalog). Second consumer: out-of-process SDK panels over `NEXUS_PANEL_SOCKET`.

## Goal

Ledger in nexus over the four surfaces; `check!` in `crates/nexus-gui` (its own workspace
root — the check runs where the arniko peer is present, like its other tests) and a
`--json` `api-drift check` step in nexus CI for the SDK-panel contract.

## Scope

- in: four surfaces, ledger, gui `check!`, CI step; NEXUS-057 and NEXUS-059 recorded as the
  first entries (Added op → Compatible; moved module → Breaking with `RenameCall`).
- out: bro-cli (stays APIDRIFT-8b/9); the GUI itself.

## Acceptance

- [ ] A `WireKey` variant or broker op added without a note fails nexus tests.
- [ ] nexus-gui's `check!` fails when it uses a key path the pinned snapshot no longer has.
- [ ] The three in-flight PRs' surface moves appear as three ledger entries after merge.
