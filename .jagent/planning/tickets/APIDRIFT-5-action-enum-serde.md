# APIDRIFT-5 — Suggestion::action enum + Item.attrs + serde + snapshot format v1

| Field | Value |
|-------|--------|
| **Status** | ready |
| **Milestone** | M2 |
| **Size** | S–M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

`Suggestion.action` is a free string; downstream cannot match on it. `Item`
carries no attributes (`non_exhaustive`, `deprecated`) that change severity.
Nothing is serializable; there is no committed snapshot file format.

## Goal

`SuggestionAction` enum with structured fields (`RenameCall { from, to }`,
`AddMatchArm { enum_path, variant }`, `ReviewSignature { old, new }`,
`RemoveItem`, `NoOp`; keep `detail` as the rendered form); `Item.attrs:
Vec<String>`; `serde` feature deriving on snapshot/break/suggestion; snapshot
file format v1 with content hash.

## Scope

- in: enum + attrs + serde derives + format doc + hash
- out: `Surface` trait / ledger crate (APIDRIFT-6)

## Acceptance

- [ ] Enum covers all current string actions (`rename-call`, `review-rename`,
      `remove-item`, `review-signature`, `review-kind`, `no-op`,
      `add-match-arm`, `review-field`)
- [ ] attrs flow from producer into classify severity
- [ ] Snapshot v1 round-trips with a verifiable content hash
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Design doc `docs/DESIGN-embedded-ledger.md` § Layers (serialization row).
Depends on APIDRIFT-4 (landed).
