# api-drift — State

**Role:** API drift handler: rustdoc-JSON producer → snapshot → diff → classify → suggest.
Core pure std, zero deps; `producer` feature adds serde_json. I/O only in the producer.
Design direction: embedded contract ledger (`docs/DESIGN-embedded-ledger.md`, proposed).

**Status (2026-09-14):** M0 + M1 complete incl. APIDRIFT-4 review hardening; M2 ticketed (5→6→8+9→7→10→11).

**License:** MIT OR Apache-2.0.

## Consumers

None yet. First candidates: real arniko rustdoc-JSON snapshots + a call-site codemod.

## Invariants

- `diff` indexes by `Item::path`; same path+sig+kind = unchanged (not reported).
- Duplicate paths: last item wins **on both sides** (old-side `Changed` branch
  guarded by `is_winner`; APIDRIFT-4 regression test).
- `classify`: `Added` = Compatible, except `Variant`/`Field` = Warning (match
  arms / struct literals); `Removed` = Breaking; fn/method `SignatureChanged`
  = Breaking unless the widening allowlist hits (else Compatible); other kinds
  = Warning; kind change = Breaking. `ClassifiedBreak` carries `item_kind`.
- `suggest`: rename folds only on same parent + same kind + signal
  (sig-modulo-leaf → auto `rename-call`; edit distance ≤ 8 → non-auto
  `review-rename`); Added `Variant` → `add-match-arm`, Added `Field` →
  `review-field` (both non-auto); everything else as before.
- Producer: public + in-crate (`crate_id == 0`) + known `inner` kinds only; unknown shapes skipped, never error.
- Core stays serde-free: `serde_json` only under `producer` (+ dev-deps for the demo test).
