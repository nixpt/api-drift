# Work Log — api-drift

```markdown
## [YYYY-MM-DD] - [Task Title] [Status Emoji]

### 📝 Summary
[Brief description of what was done]

### 📂 Files Modified
- [List of files created or changed]

### 🔗 References
- [Links to RFCs, knowledge docs, etc.]
```

## [2026-09-14] - M0 scaffold landed ✅

### 📝 Summary
Scaffolded `api-drift` as a small Rust lib (pid-event-policy shape) from
`nixpt-common/templates/rust`: snapshot/diff/classify/suggest core, zero deps.

### 📂 Files Modified
- `Cargo.toml`, `rustfmt.toml`, `.cargo/config.toml`, `.gitignore`, `.gitattributes`
- `scripts/rust-dev.sh`, `src/{lib,snapshot,diff,classify,suggest}.rs`
- `README.md`, `STATE.md`, `CLAUDE.md`, `.jagent/` planning tree

## [2026-09-14] - APIDRIFT-2 producer + demo landed ✅

### 📝 Summary
`producer` feature (serde_json-gated): rustdoc JSON → ApiSnapshot, validated
live against format v61 (55 public items). `tests/arniko_demo.rs` end-to-end.

### 📂 Files Modified
- `src/producer.rs`, `tests/arniko_demo.rs`, `Cargo.toml` (feature + dev-deps)

## [2026-09-14] - APIDRIFT-6 Surface + api-drift-ledger + dogfood ✅

### 📝 Summary
Converted the repo to a virtual workspace: `crates/api-drift` (core, moved
from `src/`) + new `crates/api-drift-ledger`. Core gains a pure-std
`surface` module (`Surface` trait, `EnumSurface`, `enum_surface!`).
Ledger crate ships `LedgerEntry`/`append_entry`/`read_entries`/`is_recorded`/
`corroboration`/`record`/`check` + the `ledger!` test macro (surface list
`;`-terminated so a nested `enum_surface!` doesn't trip `macro_rules!`).
Dogfood: `crates/api-drift/tests/dogfood.rs` embeds a ledger over
`snapshot::ItemKind` with committed `api-drift/item-kind.snapshot.json` +
`ledger.jsonl`. Folds APIDRIFT-12's antibody-note + corroboration. Fetched +
fast-forwarded upstream (release/publish/branding/docs) and reconciled
Cargo.toml workspace conversion against it.

### 📂 Files Modified
- `Cargo.toml` (→ virtual workspace), `Cargo.lock`
- `src/*` → `crates/api-drift/src/*` (+ new `surface.rs`), `tests` → `crates/api-drift/tests/` (+ `dogfood.rs`)
- `crates/api-drift-ledger/{Cargo.toml,src/lib.rs}` (new)
- `api-drift/{item-kind.snapshot.json,ledger.jsonl}` (dogfood, new)
- planning tree

### 🔗 References
- `docs/DESIGN-embedded-ledger.md` § "The ledger test, concretely" + Evorium borrows


## [2026-09-14] - APIDRIFT-13 checkstand borrow survey 📋

### 📝 Summary
Audited `projects/checkstand` (same universe). It already *practises* the
api-drift discipline by hand: `StoreError` (18 variants) + `DomainEvent` (9,
`#[serde(tag="type")]`) are enum surfaces, 7 adapters are consumers, and its
`AGENTS.md` rule #2 is literally "new variant = compile error in every adapter,
exhaustive match, no `_`". Filed APIDRIFT-13: borrow the real-server test bar
(rule #3) → APIDRIFT-9's `syn` usage scan, borrow `schemars::JsonSchema` tool
args → APIDRIFT-8 schema-hash sig, reframe `Unrecorded` note as a decision
record, and record checkstand-core as a third ledger breadboard.

### 📂 Files Modified
- `docs/DESIGN-embedded-ledger.md` (new section)
- `.jagent/planning/tickets/APIDRIFT-13-checkstand-breadboard.md` (new)
- `.jagent/planning/{STATE,TASKS}.md`

### 🔗 References
- `projects/checkstand` (`AGENTS.md`, `crates/checkstand-core`, `crates/checkstand-mcp`)

### 📝 Follow-up (same day): checkstand the other way around

checkstand's product catalog is periodic **data** (`Product { id, name,
price_cents, stock }`), re-typed independently in seven `main.rs` files with
no catalog-of-record. Because `Surface` is open, a `catalog` surface feeds
product rows through the same snapshot→diff→classify→suggest path — new SKU =
Added, re-price = SignatureChanged (Warning), delist = Removed (Breaking),
`attrs=["deprecated"]` for delist-soon. Filed APIDRIFT-14 (needs `ItemKind::
Row` + `CatalogSurface`).

## [2026-09-14] - APIDRIFT-14 data/catalog surface ✅

### 📝 Summary
Landed `ItemKind::Row` + `CatalogSurface` + `catalog_surface!`: data catalogs
(checkstand products) flow through the identical pipeline. New row = Added/
Compatible, re-price = SignatureChanged/Warning, delist = Removed/Breaking,
`attrs=["deprecated"]` for delist-soon — all falling out of existing
classify arms (no classify change needed).

### 📂 Files Modified
- `crates/api-drift/src/{snapshot.rs, surface.rs, lib.rs}`
- planning tree



## [2026-09-14] - APIDRIFT-12 Evorium borrow survey 📋

### 📝 Summary
Audited `projects/evorium` for mechanisms the embedded ledger can borrow.
Filed APIDRIFT-12 (woven into 6/9/11, no code lift — OCPL-1.1 vs MIT/Apache)
+ added a "Prior art within the fleet" section to the design doc: antibody
ledger notes w/ corroboration, autoimmune_guard for auto-apply, TreeOfCode
relatedness for gate ranking, structural-similarity rename metric (evaluate),
signed did:key notes (deferred), canopy workflow naming.

### 📂 Files Modified
- `docs/DESIGN-embedded-ledger.md` (new section)
- `.jagent/planning/tickets/APIDRIFT-12-evorium-borrows.md` (new)
- `.jagent/planning/{STATE,TASKS}.md`

### 🔗 References
- `projects/evorium/crates/{evorium-immune,evorium-genome,evorium-forge}`

## [2026-09-14] - APIDRIFT-5 action enum + serde + snapshot v1 ✅

### 📝 Summary
`SuggestionAction` enum (8 frozen verbs) replaces the string action, carrying
structured fields (`RenameCall{from,to}`, `AddMatchArm{enum_path,variant}`,
`ReviewSignature{old,new}`, `ReviewField{field}`). `Item.attrs` + producer
`collect_attrs` (`deprecated`/`non_exhaustive`); classify downgrades
Variant/Field additions under those markers. New `snapshot_file.rs` (`serde`
feature): format v1 envelope + sha256 content hash, fail-closed verification.
Feature graph: `serde` = serde+serde_json+sha2+hex; `producer` implies `serde`.

### 📂 Files Modified
- `src/suggest.rs` (enum), `src/snapshot.rs` (attrs), `src/classify.rs`
  (serde + attr severity), `src/producer.rs` (collect_attrs),
  `src/snapshot_file.rs` (new), `src/lib.rs`, `Cargo.toml`,
  `tests/arniko_demo.rs`, planning tree

### 🔗 References
- `docs/DESIGN-embedded-ledger.md` § Layers (serialization row)

## [2026-09-14] - APIDRIFT-4 core correctness ✅

### 📝 Summary
Follow-up from the captain+foreman design review (`docs/DESIGN-embedded-ledger.md`,
via `.jagent/sync/api-drift.jsonl`): three reproduced core bugs fixed and filed
as APIDRIFT-4…11 per the doc's suggested breakdown (order 4→5→6→8+9→7→10→11).
APIDRIFT-4 (this entry): old-side dupes honour last-wins in the Changed branch
(`is_winner` guard, O(n²) scan gone); `ClassifiedBreak.item_kind` added and
rename requires same kind + signal (modulo-leaf auto / edit-distance ≤ 8
`review-rename` / else separate); fn/method `SignatureChanged` → Breaking
unless the widening allowlist hits; Added Variant/Field → Warning with
`add-match-arm`/`review-field`.

### 📂 Files Modified
- `src/diff.rs`, `src/diff_tests.rs`, `src/classify.rs`, `src/suggest.rs`
- `.jagent/planning/tickets/APIDRIFT-{4,5,6,7,8,9,10,11}-*.md`
- `.jagent/planning/{STATE,TASKS}.md`, `.jagent/TODO.md`

### 🔗 References
- `docs/DESIGN-embedded-ledger.md` (proposed; agent owns the board)
- `.jagent/sync/api-drift.jsonl` (review note from nixp, 2026-09-14T22:20:37Z)

