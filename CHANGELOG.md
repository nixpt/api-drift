# Changelog

All notable changes to api-drift are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/) and versions follow
[Semantic Versioning](https://semver.org/); each released version matches a
`v*` git tag.

## [Unreleased]

### Added

- `producer` (APIDRIFT-21): **re-export materialization** — public `use` items
  now surface at their reachable local path. Named item re-exports emit one
  alias, module aliases expand their public contents (with a duplicate-segment
  collapse for `pub use zorro_io as io` over a crate that also has an `io`
  module), globs expand into the parent module (including globs of private
  local modules via the index tree), and local `use` chains are resolved.
  Aliases carry `snapshot::RE_EXPORT_ATTR` and a stable signature.
- `producer`: **signature normalization** — rustdoc ids in `resolved_path`,
  `impls`/`implementations`/`items`/`fields`/`variants`/`links` lists, and
  generic params no longer leak into signatures. Type paths prefer the shortest
  local re-export spelling when one exists.

### Fixed

- `classify`: a signature change on a re-export-backed (`re_export`) item is a
  `Warning`, not `Breaking` — the defining crate owns that shape.
- `api-drift-ledger`: `API_DRIFT_UPDATE=1` now rewrites an **existing**
  snapshot and reports `Updated`; previously it only handled the first record.

## [0.1.0] - 2026-09-14

First release: the I/O-free core plus an opt-in rustdoc-JSON producer and an
opt-in serialization layer.

### Added

- `snapshot`: `ApiSnapshot`, `Item`, `ItemKind`; items carry optional
  attribute markers (`attrs`, e.g. `deprecated`, `non_exhaustive`) that
  producers observe and `classify` consumes.
- `diff`: `diff_snapshots` → `ApiDiff { added, removed, changed }`, indexed by
  item path, output sorted by path. Duplicate paths: last item wins on both
  sides.
- `classify`: `classify_diff` → `ClassifiedBreak` with `BreakKind`,
  `Severity`, and the item kind. Rules: additions are `Compatible` except new
  enum variants and struct fields (`Warning`, unless the parent is
  `non_exhaustive` or the item is `deprecated`); removals are `Breaking`;
  function and method signature changes are `Breaking` unless they match the
  widening allowlist (`&str` → `impl Into<String>`, `&T` → `impl AsRef<T>`,
  …); kind changes are `Breaking`.
- `suggest`: `suggest_for_breaks` → `Suggestion` with a structured
  `SuggestionAction` (`RenameCall`, `ReviewRename`, `RemoveItem`,
  `ReviewSignature`, `ReviewKind`, `AddMatchArm`, `ReviewField`, `NoOp`) and
  an `auto_appliable` flag. Rename detection requires the same parent module,
  the same item kind, and a signal: identical signature modulo the leaf name
  gives an auto-appliable `RenameCall`; a small edit distance gives a
  review-only `ReviewRename`.
- `producer` feature: `snapshot_from_rustdoc_json` / `snapshot_from_rustdoc_str`
  turn a `cargo +nightly rustdoc --output-format json` file into an
  `ApiSnapshot` (in-crate public items; unknown node shapes are skipped, never
  an error; built against rustdoc JSON format v61).
- `serde` feature: serde derives on all public types, snapshot file format v1
  (`api-drift/snapshot/v1` envelope with a sha256 content hash that is
  verified on parse), `render_snapshot_file`, `parse_snapshot_file`,
  `to_string_pretty`, `content_hash`.
- `tests/arniko_demo.rs`: an end-to-end demo of a synthetic
  `arniko 0.2.98 → 0.2.99` change through the whole pipeline.

[Unreleased]: https://github.com/nixpt/api-drift/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nixpt/api-drift/releases/tag/v0.1.0
