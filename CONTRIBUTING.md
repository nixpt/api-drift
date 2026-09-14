# Contributing to api-drift

api-drift answers three questions about a Rust crate's public surface:
**what changed, how bad is it for a downstream, and what edit fixes it.** The
core is a pure-std library with no I/O; producers and consumers sit behind
cargo features or in sibling crates. Keep it that way.

## Build and test

```sh
cargo test                                   # core, zero dependencies
cargo test --all-features                    # + producer + serde + the demo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

CI runs exactly these on stable, plus `cargo check --all-features` on the
minimum supported Rust version declared in `Cargo.toml` (`rust-version`) and a
`cargo publish --dry-run`. A pull request is mergeable when all of them pass.

`rustfmt.toml` carries two nightly-only options; stable rustfmt warns and
ignores them. The formatting that counts is stable `cargo fmt`, which is what
CI checks. Running `cargo +nightly fmt` is fine locally but do not commit
import-granularity changes stable would undo.

`scripts/rust-dev.sh` is optional: it exports a per-agent `CARGO_TARGET_DIR`
when the nixpt fleet tooling is present and falls back to plain cargo defaults
otherwise. Plain `cargo test` needs nothing.

## Layout

| Path | What it is | Touch it when |
|---|---|---|
| `src/snapshot.rs` | `ApiSnapshot` / `Item` / `ItemKind` | adding a field every producer must fill |
| `src/diff.rs` (+ `diff_tests.rs`) | `diff_snapshots` | changing what counts as changed |
| `src/classify.rs` | `BreakKind` / `Severity` rules | changing severity policy (see invariants) |
| `src/suggest.rs` | `Suggestion` / `SuggestionAction` / rename detection | adding an action a consumer can apply |
| `src/producer.rs` | rustdoc JSON → snapshot (`producer` feature) | rustdoc format moves |
| `src/snapshot_file.rs` | snapshot file format v1 (`serde` feature) | never silently: bump the format string |
| `tests/arniko_demo.rs` | end-to-end demo | any of the above changes visible output |
| `docs/DESIGN-embedded-ledger.md` | where this is going (embedded ledger, surfaces, consumer checks) | before starting a ticket from it |
| `.jagent/planning/` | roadmap, tickets `APIDRIFT-NN`, state | every change of scope |

## Invariants that tests protect

- `diff` indexes by `Item::path`; identical path, kind, and signature is not
  reported. Duplicate paths: last item wins on both sides.
- `classify` errs toward **Breaking**: a function or method signature change is
  Breaking unless the widening allowlist proves source compatibility. Adding a
  variant or field is a Warning. Loosening any of this needs a test showing
  why the old label was wrong.
- `suggest` marks `auto_appliable` only when a script could apply the edit
  without judgment (a rename with a matching signature, a pure addition).
  Anything a consumer might auto-apply must be conservative first; false
  confidence here rewrites someone's call sites wrongly.
- Snapshot file format: `format` must equal `api-drift/snapshot/v1`, items are
  sorted by path on render, and a content-hash mismatch on parse is an error.
  A new field means a new format string.
- The default feature set has zero dependencies. New dependencies go behind a
  feature.

## Branches, tickets, commits

- Tickets are `APIDRIFT-NN` under `.jagent/planning/tickets/`; the board is
  `.jagent/planning/TASKS.md`. Pick or file a ticket before large work.
- Work on a branch (`agent/<name>/APIDRIFT-NN` inside the fleet, anything
  descriptive outside it) and open a pull request against `main`.
- Commit subjects use conventional prefixes (`feat:`, `fix:`, `docs:`,
  `test:`, `chore:`, `ci:`) with the ticket in the subject or body.
- Architectural decisions are recorded with [dejavue](https://github.com/nixpt/dejavue)
  (`dejavue decision …`); `.dejavue/` is committed and merges append-only.

## Never commit

`.gitignore` blocks these; the rule matters more than the mechanism because
this is a public repository:

- `.jagent/sync/` — the live agent coordination channel (per checkout).
- `.jagent/*mcp*.json`, `/.mcp.json` — fleet MCP configs can carry live
  credentials.
- `*.env`, `*.pem`, `*.key`, `credentials*.json`, `.netrc`.

## License

api-drift is dual-licensed under MIT or Apache-2.0, at your option. Unless you
state otherwise, any contribution you submit is licensed the same way, without
additional terms.
