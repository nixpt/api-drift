# APIDRIFT-4 — core correctness (design-review prerequisites)

| Field | Value |
|-------|--------|
| **Status** | done |
| **Milestone** | M1 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

2026-09-14 captain+foreman review reproduced three core bugs against the live
code, all filed in `docs/DESIGN-embedded-ledger.md` § "Changes to the core":

1. `diff`: old-side duplicate paths do not honour last-wins in the `Changed`
   branch (O(n²) `changed.iter().any` dedupe compared stale dupes).
2. `suggest`: removed `Field` + added `Method` under one parent auto-applied
   as `rename-call` (`ClassifiedBreak` carried no `ItemKind`).
3. `classify`: new required fn arg (`f(x: u8)` → `f(x: u8, y: u8)`) was
   `Warning`; Rust has no default args so it is `Breaking`.

The ledger (APIDRIFT-6+) will auto-apply what `suggest` marks
`auto_appliable`, so the core had to fail closed first.

## Goal

Conservative core: last-wins everywhere, kind-checked renames with a
similarity signal, `SignatureChanged` on fn/method defaults to `Breaking`
with a widening allowlist, added `Variant`/`Field` warn with match-arm/field
suggestions.

## Scope

- in: `diff` old-side winner guard; `item_kind` on `ClassifiedBreak`;
  kind-checked rename (sig-modulo-leaf confident / edit-distance ≤ 8
  `review-rename` / else separate); widening allowlist; Variant/Field
  Added → Warning + `add-match-arm`/`review-field`
- out: `Suggestion::action` enum + `Item.attrs` + serde/snapshot format
  (APIDRIFT-5); `Surface`/`ledger!` (APIDRIFT-6)

## Acceptance

- [x] Repro tests for all three bugs (now regression tests)
- [x] `cargo test` green (core 21+1; `--features producer` 25+1+1)
- [x] `cargo clippy --all-targets -- -D warnings` green ± feature

## Notes

Design doc: `docs/DESIGN-embedded-ledger.md`. Suggested-track name APIDRIFT-4
per the doc's breakdown; this is the first of that sequence to land.

## Resolution

- `diff.rs`: `Changed` branch guarded by `is_winner(old_by_path, item)`;
  replaced the O(n²) `.any` scan (also fixes the perf note from the review).
- `classify.rs`: `ClassifiedBreak.item_kind`; fn/method `SignatureChanged` →
  `Breaking` unless `is_widening` allowlist hits (`&str→impl Into<String>` etc.,
  whitespace-insensitive); Added `Variant`/`Field` → `Warning`.
- `suggest.rs`: rename requires same parent + same kind + signal
  (sig-equal-modulo-leaf → auto `rename-call`; edit distance ≤ 8 → non-auto
  `review-rename`); Added `Variant` → `add-match-arm`, Added `Field` →
  `review-field` (both non-auto).
- Demo note: `Alert::new` widening now classifies `Compatible` (was `Warning`)
  but still suggests `review-signature` — classification vs. suggestion
  severities are intentionally separate; the auto gate reads the suggestion.
