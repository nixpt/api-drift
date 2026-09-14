---
name: api-drift
purpose: API drift handler — snapshot a crate's public surface, diff versions, classify breaks, suggest downstream patches; heading toward an embedded contract ledger.
dcp: DCP/1.0
---

# Context

<!-- The DCP instruction layer: what an agent should *do* in this repo.
     Source of truth — adapters (CLAUDE.md / AGENTS.md / …) are generated
     from this file via `dejavue export --target <tool>`. -->

## Operating Rules

- Run `dejavue context` before changes; capture decisions with `dejavue decision --reason …`.
- **Backlog:** `.jagent/planning/ROADMAP.md` + `tickets/` (`APIDRIFT-NN`); board in `TASKS.md`. Design direction: `docs/DESIGN-embedded-ledger.md` (ratified idea, tickets 6→11).
- **Public repo posture:** MIT OR Apache-2.0, crates.io name `api-drift`. Never commit `.jagent/sync/`, MCP configs, or anything secret-shaped. No box paths (`/build`, `/workspace`) in tracked docs.
- **Core stays pure std, zero deps, no I/O.** New dependencies go behind a cargo feature (`producer`, `serde`); new I/O goes in a feature module or a sibling crate.
- **Conservative labels:** `classify` errs toward `Breaking`; `suggest` marks `auto_appliable` only for edits a script can apply without judgment. Loosening either needs a test proving the old label wrong.
- Snapshot file format changes bump the `format` string; never change `api-drift/snapshot/v1` in place.
- Work on a branch, PR to `main`; `main` will carry the fleet-default ruleset (no force-push, no deletion). Run stable `cargo fmt` before committing — CI checks stable formatting.
- Coordinate live work on `scripts/api-drift-sync` (fleet-only tool; excluded from the package).

## Build / Test

- `cargo test` (core), `cargo test --all-features` (+ producer, serde, `tests/arniko_demo.rs`)
- `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`
- MSRV 1.74 (`[lints]` table); `Cargo.lock` stays format v3 so the MSRV job can read it
- Producer input: `cargo +nightly rustdoc -- -Z unstable-options --output-format json`

## Architecture Map

- `src/snapshot.rs` — `ApiSnapshot` / `Item` (path, kind, sig, attrs) / `ItemKind`
- `src/diff.rs` (+ `diff_tests.rs`) — `diff_snapshots`: indexed by path, last duplicate wins on both sides, sorted output
- `src/classify.rs` — `BreakKind` / `Severity` / widening allowlist / `non_exhaustive`+`deprecated` handling
- `src/suggest.rs` — `Suggestion` / `SuggestionAction` (8 structured verbs) / rename detection (same parent + same kind + signal)
- `src/producer.rs` — rustdoc JSON → snapshot (`producer` feature; rustdoc format v61; unknown nodes skipped)
- `src/snapshot_file.rs` — snapshot file format v1 with sha256 content hash, fail-closed parse (`serde` feature)
- `tests/arniko_demo.rs` — synthetic `arniko 0.2.98→0.2.99` through the whole pipeline
- `docs/DESIGN-embedded-ledger.md` — next shape: `Surface` trait, `ledger!` test macro, committed snapshot + `ledger.jsonl`, consumer pins; first customer bro-acp / bro-desktop (fleet-private repos)

## Memory

Decisions, blockers, and constraints are captured in `.dejavue/` — run
`dejavue context` for the boot packet and `dejavue recall <query>` to search.
