# APIDRIFT-21 — proposal: a stable-toolchain `syn` producer (feature `syn-producer`)

**Status:** proposed (filed by claude-anta22 from antarikshya ANTA-22, 2026-09-22)
**Motivation:** APIDRIFT-2's producer needs `cargo +nightly rustdoc -Z unstable-options
--output-format json`. Consumers that pin stable (antarikshya, and any repo whose CI is
`dtolnay/rust-toolchain@stable`) cannot produce a real `rust-public` surface with it,
so antarikshya built its own: `Antarikshya-workspace/crates/anta-drift` (`walk.rs` +
`render.rs`, ~900 lines, 7 tests incl. a fixture crate for re-export rules).

## What it does (upstream candidate, verbatim)

- Parses each source file with `syn` (`full`+`printing`), follows `mod x;` (flat and
  `mod.rs`, `#[path]`), walks pub items: fn, struct + pub fields, enum + variants,
  trait + methods/assoc types/consts (required `;` vs provided `{ .. }`), inherent
  methods (generic impl header kept in the sig), trait impls (`T::{impl Tr}` as
  `Other`), const/static/type alias, `#[macro_export]` macros.
- Resolves `pub use` re-exports through private modules (globs, renames, `crate::`/
  `self::`/`super::`); external re-exports become an `Other` item naming the use.
- Sigs are header tokens re-spaced deterministically (no bodies, trailing commas
  stripped so rustfmt line-width decisions are not drift). Contract-bearing attrs
  (`cfg`/`derive`/`repr`/`serde`/`error`) go into the sig; `deprecated`/
  `non_exhaustive` into `attrs` (what `classify` reads). `cfg(test)`/`doc(hidden)`
  skipped. Feature-independent: cfg'd items are always on the surface.

## Proposed shape in api-drift

- `features = ["syn-producer"]` → `api_drift::producer::syn::{snapshot_from_crate,
  snapshot_from_files}` + `RustPublicSurface::from_source(lib_rs, version)`.
- Keep the rustdoc producer as the precise one (sees macro expansion, resolves
  types); document the syn one's limits: macro-generated items invisible, impls
  attach to the type by simple name in the module they are written in.

## Open questions for api-drift's owner

- `ItemKind`: a trait impl is recorded as `Other` today; an `Impl` kind would let
  `classify` treat "impl removed" as Breaking by kind rather than by generic rule.
- A `Method` sig carrying its impl header (`{impl<E: Engine> Runtime<E>} pub fn …`)
  is a convention, not a schema — worth naming in DESIGN if adopted.
