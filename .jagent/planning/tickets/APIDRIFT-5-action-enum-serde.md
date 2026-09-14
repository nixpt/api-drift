# APIDRIFT-5 — Suggestion::action enum + Item.attrs + serde + snapshot format v1

| Field | Value |
|-------|--------|
| **Status** | done |
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

- [x] Enum covers all current string actions (`rename-call`, `review-rename`,
      `remove-item`, `review-signature`, `review-kind`, `no-op`,
      `add-match-arm`, `review-field`) — `action_names_frozen` test
- [x] attrs flow from producer into classify severity
- [x] Snapshot v1 round-trips with a verifiable content hash
- [x] `cargo test` green: 26 base / 30 serde / 36 producer / 36 both, +1 doctest; clippy `-D warnings` clean on all four

## Notes

Design doc `docs/DESIGN-embedded-ledger.md` § Layers (serialization row).
Depends on APIDRIFT-4 (landed).

## Resolution

- `suggest.rs`: `SuggestionAction` enum (8 variants, frozen `name()` strings,
  `action_names_frozen` guards renames) replaces the string `action`;
  `RenameCall`/`ReviewRename` carry `{from,to}`, `AddMatchArm` carries
  `{enum_path, variant}` split from the break path, `ReviewSignature`
  carries `{old,new}`, `ReviewField` carries `{field}`.
- `snapshot.rs`: `Item.attrs: Vec<String>` + `with_attr`/`with_attrs`/
  `has_attr`; serde derives on `ItemKind`/`Item`.
- `classify.rs`: serde derives on all three enums/structs; Added
  `Variant`/`Field` downgrade to Compatible when `deprecated` or
  `non_exhaustive`; deprecated removals noted in the break note.
- `producer.rs`: `collect_attrs` reads rustdoc `deprecation` (non-null) +
  `attrs` list (string or object encoding) → `deprecated`/`non_exhaustive`
  markers; `attrs_flow_from_nodes` regression test.
- `snapshot_file.rs` (new, `serde` feature): `SnapshotFile` envelope
  (`api-drift/snapshot/v1` + surface + version + sorted items +
  `content_hash`), `content_hash` = sha256 over `path\0kind\0sig\0sorted-
  attrs\0` lines, `parse_snapshot_file` fails closed on format mismatch or
  hash mismatch; round-trip / tamper / wrong-format / order-stability tests.
- Feature graph: `serde` = serde + serde_json + sha2 + hex; `producer` now
  implies `serde` (producer already needed serde_json). Core (no features)
  stays pure std.

