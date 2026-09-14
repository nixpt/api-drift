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

