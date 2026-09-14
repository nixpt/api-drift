# api-drift

API drift handler: snapshot a crate's public surface, diff versions, classify
breaks, suggest downstream patches.

**Motivation:** `arniko` is consumed by many sibling repos. When an upstream
API changes (rename, new required arg, removed item), downstream crates break
and someone hand-edits every call site. `api-drift` is the policy/type layer
for that workflow — the mechanical part (what changed, how bad, what edit
fixes it) so agents/humans stop rediscovering it per repo.

## Pipeline

```text
producer ──▶ ApiSnapshot ──▶ diff ──▶ classify ──▶ suggest ──▶ consumer
(rustdoc JSON,   versioned     ApiDiff   ClassifiedBreak  Suggestion   (codemod,
 `cargo            inventory                                     rename/add-arg/  agent,
 public-api`,                                                 review-*)  human)
 tree-sitter…)
```

The snapshot/diff/classify/suggest core is I/O-free (pure std, zero deps).
The rustdoc-JSON producer lives behind the `producer` feature (`serde_json`).

## API

```rust,ignore
use api_drift::{
    snapshot::{ApiSnapshot, Item, ItemKind},
    diff::diff_snapshots, classify::classify_diff, suggest::suggest_for_breaks,
};

let old = ApiSnapshot::new("arniko 0.2.98", vec![
    Item::new("arniko::Alert::new", ItemKind::Function, "pub fn new(message: &str) -> Self"),
]);
let new = ApiSnapshot::new("arniko 0.2.99", vec![
    Item::new("arniko::Alert::new", ItemKind::Function, "pub fn new(message: impl Into<String>) -> Self"),
]);

let diff = diff_snapshots(&old, &new);      // added / removed / changed
let breaks = classify_diff(&diff);          // + BreakKind + Severity
let fixes = suggest_for_breaks(&breaks);    // rename-call / review-signature / …
```

Rename detection: a `Removed` + an `Added` under the same parent module
collapse into one auto-appliable `rename-call` suggestion.

## Producer (`producer` feature)

```bash
cargo +nightly rustdoc -- -Z unstable-options --output-format json
```

```rust,ignore
let snap = api_drift::producer::snapshot_from_rustdoc_json(
    std::path::Path::new("target/doc/arniko.json"), "arniko 0.2.99",
)?;
```

Validated against real rustdoc JSON (format v61, nightly-1.100) on this very
crate: `cargo +nightly rustdoc -- -Z unstable-options --output-format json`
then parse — 55 public items round-tripped through the producer in the probe
(not a committed test; rustdoc format drifts per toolchain). Committed:
`tests/arniko_demo.rs` — synthetic arniko 0.2.98→0.2.99 rustdoc docs through
the full pipeline (widened `Alert::new`, `badge`→`tag` rename, removed
`legacy`, added `Sparkline::render`):

```text
[Warning/SignatureChanged] arniko::Alert::new
[Compatible/Added] arniko::Sparkline::render
[Breaking/Removed] arniko::badge → suggest rename-call (auto)
[Breaking/Removed] arniko::legacy → suggest remove-item (review)
```

## Layout

```text
api-drift/
├── Cargo.toml            crate api-drift (core zero-dep; producer feature)
├── src/
│   ├── lib.rs            pipeline docs + re-exports
│   ├── snapshot.rs       ApiSnapshot / Item / ItemKind
│   ├── diff.rs           ApiDiff / diff_snapshots
│   ├── classify.rs       BreakKind / Severity / classify_diff
│   ├── suggest.rs        Suggestion / suggest_for_breaks (+rename detect)
│   └── producer.rs       rustdoc-JSON → ApiSnapshot (`producer` feature)
├── tests/arniko_demo.rs  end-to-end demo (requires `producer` feature)
├── .jagent/              planning board (APIDRIFT-NN tickets)
└── STATE.md              delivery snapshot
```

## Build / test

```bash
source scripts/rust-dev.sh   # per-agent CARGO_TARGET_DIR (+ sccache)
cargo test                             # core: 15 tests + 1 doctest
cargo test --features producer         # + 4 producer tests + demo test
cargo clippy --all-targets -- -D warnings
```

## Status

M0 scaffold + M1 producer (APIDRIFT-1/2 done). See `STATE.md` and
`.jagent/planning/` for the roadmap (CLI, downstream appliers).

## License

MIT OR Apache-2.0
