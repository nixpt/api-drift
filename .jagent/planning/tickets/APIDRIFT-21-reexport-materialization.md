# APIDRIFT-21 — Producer: re-export materialization + id normalization

| Field | Value |
|-------|-------|
| **Status** | done |
| **Milestone** | M1 (producer hardening) |
| **Size** | M |
| **Owner** | opencode |
| **Created** | 2026-10-22 |
| **Updated** | 2026-10-22 |
| **Driver** | ZORRA-1 step 4 (zorro 13-crate decomposition gate) |

## Problem

Two producer blindspots made the ZORRA-1 api-drift ledger unusable beyond the
consumer-compile check:

1. **Re-exports vanished.** A `pub use` surfaces nothing in the snapshot, so
   the post-split `zorro::io`, `zorro::kv_cache`, `zorro::pipeline::sampling`
   surfaces were reported as `Removed` even though they exist.
2. **rustdoc ids leaked into signatures.** Ids renumber on every rebuild and
   across a crate split, so structurally identical items appeared as
   `SignatureChanged` ("id-noise").

## What changed

- **`pub use` materialization** in `producer.rs`:
  - named item re-exports emit one alias at `<parent>::<name>`;
  - named module aliases expand the module's public contents
    (`pub use zorro_io as io;`), with a duplicate-segment collapse so a crate
    that also has an `io` module maps to `zorro::io::X`, not `zorro::io::io::X`;
  - globs expand into the parent module, including globs of **private local
    modules** (walked through the index tree, which `paths` cannot express);
  - local alias chains (`pub use a::X` where `a::X` is itself a local `use`)
    are resolved to the ultimate target.
  - Aliased items carry [`snapshot::RE_EXPORT_ATTR`] and a stable signature
    (target payload for in-crate targets; `re-export of <path>` for external
    ones).
- **Signature normalization**:
  - `resolved_path` uses the defining crate's canonical path and drops its id;
  - `impls` / `implementations` / `items` / `fields` / `variants` / `links`
    id lists become sorted stable labels (paths, local names, impl
    descriptors);
  - generic-param and impl ids become `local:<name>` / `impl:<trait> for <type>`.
- **Alias-aware signature paths**: after all aliases are discovered, signature
  type paths prefer the shortest local re-export spelling, so
  `zorro_nn::backend::ComputeBackend` renders as
  `zorro::backend::ComputeBackend` when the documenting crate re-exports it.
- **classify**: a signature change on an item carrying `re_export` is
  `Warning` (the defining crate owns that shape), not `Breaking`.
- **ledger fix**: `API_DRIFT_UPDATE=1` now rewrites an existing snapshot and
  returns `Updated`; previously it only handled the first record, so the docs'
  recording command silently did nothing after init.

## Known limits (documented, not silent)

rustdoc JSON's `paths` table is a **reference** table: it lists items the
documented crate mentions, not a full index of every dependency. Therefore:

- a dependency's flattened re-exports (`zorro_io`'s `pub use io::*`) are only
  materialized where referenced; `zorro::io::X` may still be absent if nothing
  in the root's public signatures mentions it. Materializing those needs the
  dependency's own rustdoc JSON.
- a type that moved between crates during a split (`TensorIndex`:
  `zorro::io::gguf::TensorIndex` → `zorro_core::tensor_info::TensorIndex`) still
  reads as a signature change: it is genuine provenance drift, not id churn.

## Acceptance

- [x] synthetic tests for named re-export, module alias, glob, private-module
      glob, local chain, id churn (same shape + different ids → no diff),
      re-export attrs;
- [x] `cargo test --workspace --all-features` green;
- [x] `cargo clippy --workspace --all-features --all-targets` clean;
- [x] demo ledger regenerated (only the `resolved_path.id` removal);
- [x] validated against zorro: current surface 61 → 202 items, aliases visible,
      id churn gone, consumer compile still the enforced gate.
