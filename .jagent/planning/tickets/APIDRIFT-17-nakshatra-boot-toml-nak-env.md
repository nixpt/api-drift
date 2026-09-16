# APIDRIFT-17 — nakshatra: `boot.toml` keys + `NAK_*` env as catalog surfaces (C producer); nexus recipe + exo-init as consumers

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-16 |
| **Updated** | 2026-09-16 |

## Problem

On 2026-09-16 the nexus tty2 staging recipe (`nexus/docs/feasibility/exo-tty2-integration.md`,
`scripts/exo-tty2-image-stage.sh`) was found to write `tty = 2`, `rescue_tty = 1` into
`boot.toml` — keys `init/nak-init.c` does not parse (its set: `target, boot_caps,
capsule_policy, gen_*, readonly, seccomp, verity*`). Silently ignored; nobody noticed for a
week. That is a consumer referencing items the producer does not have — the precise case
`check!`'s usage scan exists to fail — in a **C** producer with no rustdoc.

NAK-16 adds the keys and forwards `NAK_TTY`/`NAK_RESCUE_TTY`; exo-init (EXO-214) consumes them.

## Goal

A `catalog_surface!` of `boot.toml` keys and one of `NAK_*` env names, written next to the
parser in a tiny Rust test crate under `nakshatra/tools/` (host tooling only — nothing enters
the boot TCB), with a test that greps `nak-init.c` for every listed key so the list and the
parser cannot disagree. Consumers pin it: the nexus recipe (a TOML/markdown key extractor) and
exosphere's exo-init (`NAK_*` string references).

## Scope

- in: surfaces + ledger in nakshatra; the grep-parity test; a non-Rust reference extractor
  (TOML keys in a script/doc, `getenv("NAK_…")` in Rust) for `check!`; NAK-16 as the first
  entry (Added, Compatible).
- out: parsing TOML in nak-init (it is a string table by design); nakshatra's diverged local
  `main` (foreman).

## Acceptance

- [ ] Listing a key the parser lacks (or vice versa) fails nakshatra's host tests.
- [ ] `api-drift check` in nexus reports the recipe's `tty`/`rescue_tty` as **unknown items**
      against the pre-NAK-16 snapshot and as Compatible against the post-NAK-16 one.
