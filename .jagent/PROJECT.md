# api-drift

## What This Is

**api-drift** (API change handler) — snapshot crate surfaces, diff versions,
classify breaks, suggest downstream patches. Kills the manual fix-every-callsite
loop when a shared crate (e.g. arniko) moves its API.

Library + (later) CLI. I/O-free core (`snapshot`/`diff`/`classify`/`suggest`);
producers parse real code, consumers apply patches.

## Core Value

Upstream API moves once; downstream fixes itself (or gets told exactly how).

## Current State

M0 scaffold: core pipeline + unit tests, `cargo test` green.
See `.jagent/planning/STATE.md` for the full delivery snapshot.

## Key Decisions

| Decision | Choice | Why |
|----------|--------|-----|
| Small Rust lib first (not CLI/workspace) | `snapshot/diff/classify/suggest` modules | Matches pid-event-policy shape; adoptable before tooling |
| I/O-free, zero deps | pure std, deny unsafe | Leaf posture like nixpt-common; producers stay outside |
| Template source | `nixpt-common/templates/rust` + pid-event-policy dots | Fleet-canonical lint/fmt/dev-script set |

Full decisions with rejected alternatives in `.dejavue/decisions.md`.
