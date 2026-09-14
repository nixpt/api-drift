# Milestone M0 — Core pipeline

**Status:** in_progress
**Exit:** snapshot → diff → classify → suggest works on hand-built snapshots, tested.

## Delivered

- [x] `ApiSnapshot` / `Item` / `ItemKind` — APIDRIFT-1
- [x] `diff_snapshots` → `ApiDiff` — APIDRIFT-1
- [x] `classify_diff` → `BreakKind` + `Severity` — APIDRIFT-1
- [x] `suggest_for_breaks` → `Suggestion` (+rename detect) — APIDRIFT-1
- [x] `cargo test` + `cargo clippy` green — APIDRIFT-1 (15 tests + 1 doctest, clippy `-D warnings` clean)

## Verification

```bash
source scripts/rust-dev.sh
cargo test
cargo clippy --all-targets -- -D warnings
```

## Versions

| Ticket | Component | Tests added |
|--------|-----------|-------------|
| APIDRIFT-1 | snapshot | 2 |
| APIDRIFT-1 | lib doctest | 1 |
