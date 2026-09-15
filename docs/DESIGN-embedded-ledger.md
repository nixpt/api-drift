# api-drift as an embedded contract ledger

> Design proposal, 2026-09-14. Captain's idea, expanded with the review
> findings from the same day. Status: **proposed** — the api-drift agent owns
> the board; the ticket breakdown at the bottom is a suggestion, not a filing.

## The idea

Today api-drift is a library you *point at* two snapshots. The bigger shape:
**any Rust project embeds api-drift as a module that records its own API
changes.** The module never touches the project's functionality. It only
observes the project's public contract, keeps a versioned ledger of that
contract inside the repo, and lets anyone (a downstream crate, an agent, a
merge gate) ask "what changed since the version I pinned, does it break me,
and what edit fixes it".

The first real customer is not arniko. It is **bro-acp**: bro-cli's ratified
session architecture (`bro-cli/docs/session-architecture.md`) makes
bro-agent an ACP server and bro-tui, bro-desktop and editors its clients.
BRO-98 rewrites `bro_tui::backend::BackendEvent` and adds `_bro/*`
extension methods. bro-desktop matches exhaustively on that enum today
(`bro-desktop/src/agent.rs:102`), so the change *will* break it. With the
ledger embedded in bro-tui and bro-desktop pinned to it, that break is
inferred before merge, with the file and line to edit.

## What "embedded" means

Three properties, in priority order:

1. **Zero runtime footprint.** The ledger runs as a `#[test]` (dev-dependency
   only), the way `insta` and `cargo-public-api`'s test pattern work. No
   `build.rs`, no runtime dep, nothing in the shipped binary. A project that
   embeds api-drift gains one test and one directory.
2. **The ledger lives in the repo.** `api-drift/<surface>.snapshot.json` is
   the current contract; `api-drift/ledger.jsonl` is the append-only history
   of classified changes with the producer's migration note. Both are
   committed, so a consumer can read them from a git dependency, a tag, or a
   sibling checkout without running anything upstream.
3. **Surfaces, not just `pub` items.** A crate's Rust API is one surface. bro
   has others that matter more to its clients: ACP methods and their JSON
   schemas, `BackendEvent` variants, MCP tool schemas, CLI flags, config
   keys. Each is a `Surface` that produces an `ApiSnapshot`; the core does
   not care which.

## Architecture

```text
                    upstream repo (e.g. bro-tui)
┌──────────────────────────────────────────────────────────────┐
│  src/…            (untouched)                                 │
│  tests/api_drift.rs   ledger!("acp-contract", AcpSurface)    │──┐ test-time
│  api-drift/                                                   │  │
│    acp-contract.snapshot.json   ← current contract            │◄─┘
│    ledger.jsonl                 ← history + migration notes   │
│    consumers.toml               ← who pins this surface       │
└──────────────────────────────────────────────────────────────┘
          ▲ reads (git dep / tag / sibling path)
┌──────────────────────────────────────────────────────────────┐
│  downstream repo (e.g. bro-desktop)                           │
│  api-drift.toml   [[upstream]] name="bro-tui" surface="acp-contract" pin="v1.2.6" │
│  tests/api_drift.rs   check!()  → breaks ∩ used items → suggestions with call sites │
└──────────────────────────────────────────────────────────────┘
          ▲ asks
   agents / foreman merge gate / `api-drift` CLI (--json)
```

### Layers

| Layer | Crate / feature | Depends on | Notes |
|---|---|---|---|
| core | `api-drift` (exists) | std | snapshot / diff / classify / suggest. Stays pure. |
| serialization | feature `serde` | serde, serde_json | `ApiSnapshot`, `ClassifiedBreak`, `Suggestion` derive; snapshot file format v1 |
| surfaces | feature `surface` | core | `trait Surface { fn name(&self) -> &str; fn snapshot(&self) -> ApiSnapshot; }` plus built-ins below |
| `rust-public` surface | feature `rust-public` | rustdoc JSON (nightly) now, `syn` on stable later | a nightly toolchain is required for rustdoc JSON today |
| `protocol` surface | feature `protocol` | schemars (optional) | items are ACP/MCP methods with request/response schema hashes as `sig`; `_bro/*` extensions first |
| `cli` surface | feature `cli` | clap introspection | subcommands and flags as items; `bro` gains this for free when BRO-112 moves the bin |
| `enum` surface | feature `surface` | none | derive-free: a macro lists variants of a public enum as `Variant` items (the `BackendEvent` case) |
| ledger | crate `api-drift-ledger` | core + serde | the `ledger!` test macro, snapshot/ledger file I/O, migration-note prompt |
| consumer | crate `api-drift-ledger` | core + serde + (usage scan) | `api-drift.toml` pins, `check!` test, usage scan of the consumer's own source for pinned paths |
| CLI | crate `api-drift-cli` (`api-drift` bin) | all | `check`, `note`, `affected`, `release-hint`, `--json` for agents |

The core stays zero-dep by default. Every producer and all I/O sit behind
features or in sibling crates, matching the leaf posture in `PROJECT.md`.

### The ledger test, concretely

```rust
// bro-tui/tests/api_drift.rs
api_drift_ledger::ledger! {
    dir: "api-drift",
    surfaces: [
        api_drift::rust_public::Surface::for_crate("bro-tui"),
        api_drift::enum_surface!(bro_tui::backend::BackendEvent),
        bro_tui::acp::contract_surface(),   // hand-written: ACP + _bro/* methods
    ],
}
```

Behaviour of the generated test:

- Regenerate each surface, diff against the committed `*.snapshot.json`.
- No diff: pass.
- Diff, and `ledger.jsonl` already has an entry whose `snapshot_hash`
  matches: pass (someone recorded it).
- Diff without an entry: **fail** with the classified breaks and the exact
  command to record them: `api-drift note --surface acp-contract "…"`. The
  note is the producer's migration hint, written at change time by the
  person who knows why. That hint is what `suggest` can never invent.
- `API_DRIFT_UPDATE=1 cargo test` records automatically with an empty note
  for local iteration; CI runs without it.

Ledger entry (one JSON object per line):

```json
{"ts":"2026-09-20T18:02:11Z","surface":"acp-contract","from":"sha256:…","to":"sha256:…",
 "version":"1.3.0","breaks":[{"path":"bro_tui::backend::BackendEvent::Partial","kind":"Added",
 "item_kind":"Variant","severity":"Warning"}],
 "note":"BRO-98: BackendEvent gained Partial/ToolCall/ToolCallUpdate/Permission; add match arms, Output still carries complete lines",
 "author":"opencode2","ref":"BRO-98"}
```

### The consumer check, concretely

```toml
# bro-desktop/api-drift.toml
[[upstream]]
name    = "bro-tui"
surface = "acp-contract"
source  = { path = "../bro-tui/api-drift" }     # or git = "…", tag = "v1.2.6"
pin     = "sha256:…"                            # snapshot hash at last known-good
```

`check!` in bro-desktop's tests reads the upstream ledger from `pin` to head,
folds the breaks, then intersects them with the paths bro-desktop actually
uses (a `syn` walk of its own `src/` for `use` paths, method calls and enum
patterns; polydex can replace this later). Output is the suggestion list
scoped to real call sites:

```text
bro-tui/acp-contract  pin 1a2b…  → head 9f8e…  (1 ledger entry, BRO-98)
  Warning  Added Variant BackendEvent::Partial        used: src/agent.rs:102 (exhaustive match)
  Warning  Added Variant BackendEvent::ToolCall       used: src/agent.rs:102
  Breaking Removed        BackendEvent::Output(String) → Output(Line)   used: src/agent.rs:103
    note: BRO-98: … add match arms, Output still carries complete lines
    suggest: add-match-arm ×3 (auto), review-signature ×1
```

That is the "breaking change for bro-desktop is inferred" moment. It runs in
bro-desktop's own `cargo test`, in the foreman merge gate before BRO-98 is
merged (the gate reads bro-tui's *branch* ledger), and from an agent via
`api-drift affected --json`.

### Fleet registry

`api-drift/consumers.toml` in the upstream lists who pins which surface. It
is advisory and self-maintained: `api-drift check` in a consumer offers to
append itself. The foreman merge gate uses it to print, for a PR touching a
ledgered surface, the list of affected downstream repos and call-site counts,
which is exactly the post-merge semantic-conflict class the merge-wave skill
warns about, caught pre-merge.

## Changes to the core that this needs (from the 2026-09-14 review)

These are prerequisites, not nice-to-haves: the ledger will auto-apply what
`suggest` marks `auto_appliable`, so the core must be conservative first.

1. `diff`: old-side duplicates must honour last-wins (guard the `Changed`
   branch with `is_winner`; drop the O(n²) `changed.iter().any`).
2. `ClassifiedBreak` carries `item_kind: ItemKind`; rename detection requires
   same kind **and** a similarity signal (same `sig` modulo the last path
   segment, or small edit distance). Otherwise `review-rename`,
   `auto_appliable: false`. Today a removed field + an added method under
   one parent auto-applies as a rename.
3. `classify`: `SignatureChanged` on `Function`/`Method` defaults to
   `Breaking` (Rust has no default args); a small widening allowlist
   (`&str → impl Into<String>`, `&T → impl AsRef<T>`, `T → impl Into<T>`)
   downgrades to `Compatible`. `Added` + `Variant` → `Warning` with an
   `add-match-arm` suggestion; `Added` + public `Field` → `Warning`.
4. `Suggestion.action` becomes an enum with structured fields
   (`RenameCall { from, to }`, `AddMatchArm { enum_path, variant }`,
   `ReviewSignature { old, new }`, `RemoveItem`, `NoOp`); `detail` stays as
   the rendered form.
5. `Item` gains optional `attrs: Vec<String>` (e.g. `non_exhaustive`,
   `deprecated`) so producers can pass what changes the severity.

## Prior art and why not just use it

- `cargo-semver-checks` / `cargo-public-api`: excellent for the Rust `pub`
  surface, nightly-bound, crate-only, no notion of a consumer's usage, no
  migration notes, no non-Rust surfaces (ACP, MCP, CLI). Neither is on the
  box. api-drift should *consume* their output as a producer if that is
  cheaper than rustdoc JSON directly, not compete with them.
- `insta`: the snapshot-test ergonomics to copy.
- dejavue: records *why* a decision was made; the ledger records *what*
  changed in a machine-readable contract. They link: a ledger entry's `ref`
  points at a ticket, a dejavue decision can cite a ledger hash.

## Prior art within the fleet: Evorium (borrowed vocabulary, 2026-09-14)

`projects/evorium` (OpenKO) models software evolution as a living system —
genome/capsule/habitat/canopy/forge/immune. Several of its concrete mechanisms
map almost one-to-one onto the embedded ledger; filed as APIDRIFT-12:

| Evorium piece | Where | Borrow |
|---|---|---|
| `immune::Antibody` + `ImmuneMemory::record/merge/consult` | ledger notes + APIDRIFT-9 `check!` | The migration note **is** an antibody: the first consumer that breaks records detection + patch guidance, every other consumer gets immunity pre-merge via `check!`. Add `corroboration: u32` to ledger notes (rises when multiple consumers confirm the same break). |
| `immune::autoimmune_guard(proposed, confidence)` | APIDRIFT-4 conservative core | Already the *shape* of our `auto_appliable` gate (weak-signal rename can never auto-apply). Formalize: `Suggestion` gains a `confidence: f64` and the ledger's auto-applier consults an `autoimmune_guard` before applying anything above Tolerate. |
| `genome::TreeOfCode::relatives` (Jaccard over ancestor sets ≥ 0.5) | APIDRIFT-11 registry/gate | "Who shares enough ancestry to develop the same disease?" = which downstream repos consume a drifted surface. Gate output ranks affected repos by relatedness, not just by list order. |
| `genome::Anatomy::structural_similarity` (cosine over counts) | rename detection | Candidate additional/alternative weak-signal metric for same-kind rename pairs (current: sig edit distance ≤ 8). Compare against the Levenshtein path before switching. |
| `forge::PatchArtifact` (signed, `did:key`, trust score, channels) | ledger entries | Ledger notes can graduate to **signed artifacts** with a `did:key` identity (interop with XIP DIDs) + peer corroboration; `--min-trust` for auto-apply. Start unsigned + advisory (immune's own caveat: unsigned antibodies are advisory), graduate when peer sync exists. |
| `forge::ManifestDelta::compute(from, to)` | snapshot diffing | Same diff-of-manifests shape; their *signed delta artifact* pattern maps to delta snapshots between pins — revisit if delta snapshots beat full-snapshot files. |
| `canopy` snapshot → plan → transition → verify → commit/restore | ledger workflow | The ledger-record cycle should mirror it: diff (plan) → verify → append entry (commit); a bad entry restores by appending a correction, never rewriting history (append-only jsonl already enforces this). |

Privacy invariant shared and adopted: observations/ledger entries carry only
failure patterns (paths, sigs, hashes) — never user data.

## Prior art within the fleet: checkstand (the manual, single-repo version)

`projects/checkstand` is the same universe, one step behind in time: it
*already* runs the drift discipline api-drift automates, but as a convention
inside one repo instead of a cross-repo, pre-merge, ledgable gate.

Its own `AGENTS.md` operating rule #2 is the thesis, verbatim:

> Every `StoreError` variant must be exhaustively matched in every adapter's
> error-mapping function — no wildcard `_` arm. Adding a variant to core is
> supposed to be a compile error in every adapter until you decide what HTTP
> status / gRPC code / GraphQL error it maps to.

checkstand-core's `StoreError` (18 variants) + `DomainEvent` (9 variants,
`#[serde(tag = "type")]`) are enum surfaces; its 7 adapter crates are the
consumers; the `From<StoreError>` mappers (e.g. `checkstand-rest`'s
`ApiError::from`) are the call-sites. api-drift turns rule #2 from a
"remember to write exhaustive matches" convention into a mechanical
`Added Variant → Warning → add-match-arm` finding, recorded in a ledger
with a migration note, reported across repositories and *before* merge.
Filed as APIDRIFT-13:

| checkstand piece | Where | Borrow |
|---|---|---|
| AGENTS rule #2 ("adding a variant is *a decision to be made*, not a paper-over") | core philosophy | Reframe `Unrecorded` drift: the recording note is the producer's decision record, not just a prompt — keep the "no catch-all `_`" spirit in the consumer check. |
| Rule #3 (never call a handler in a test — spawn the real server + real clients) | APIDRIFT-9 consumer check | The "real, not asserted" bar: usage scan must be a genuine `syn` walk of the consumer's source, and the test must drive the real check, not a fixture. |
| `schemars::JsonSchema` on MCP tool args + `#[tool(description = …)]` | APIDRIFT-8 protocol surface | Schema-hash sig: hash each tool's request/response JSON Schema. checkstand-mcp proves it's cheap and already in-fleet (every `*Args` struct derives it). |
| `#[serde(tag = "type")]` `DomainEvent` externally-tagged enum | APIDRIFT-7/8 surface targets | A wire-visible enum is a first-class `enum_surface!` target; the serde tag discriminant is the natural variant sig. |

### checkstand as a third breadboard

Beyond bro-acp (APIDRIFT-8+9) and arniko (APIDRIFT-7), checkstand-core is the
cleanest self-contained demo of the ledger: `ledger!` over `StoreError` +
`DomainEvent`; its 7 adapters are the "consumers" a registry/merge-gate
(APIDRIFT-11) would list when a variant is added. A new `StoreError` variant
becomes one ledger entry + `add-match-arm ×7` (one per adapter) instead of
seven simultaneous compile errors discovered one `cargo check` at a time.


## Non-goals

- Auto-landing patches without review (unchanged from the roadmap).
- Semantic type-aware diffing in the core.
- Runtime registration or reflection. Everything is test-time and file-based.

## Suggested ticket breakdown (agent's call to file/renumber)

| Ticket | Title | Size | Depends on |
|---|---|---|---|
| APIDRIFT-4 | Core correctness: last-wins on old side, `item_kind` on breaks, kind-checked rename with similarity, `SignatureChanged` → Breaking + widening allowlist, `Variant`/`Field` added → Warning | S–M | — |
| APIDRIFT-5 | `Suggestion::action` enum + `Item.attrs`; `serde` feature; snapshot file format v1 with content hash | S–M | 4 |
| APIDRIFT-6 | `Surface` trait + `enum_surface!` + `api-drift-ledger` crate with the `ledger!` test macro and `ledger.jsonl`; dogfood on api-drift itself | M | 5 |
| APIDRIFT-7 | `rust-public` surface via rustdoc JSON (nightly on box); arniko 0.2.98→0.2.99 demo becomes a ledger test | M | 6 |
| APIDRIFT-8 | `protocol` surface for ACP/MCP methods + JSON schema hashes; embed in bro-tui for the BRO-98 contract (coordinate with bro-cli BRO-98's horse) | M | 6 |
| APIDRIFT-9 | Consumer side: `api-drift.toml` pins, `check!`, `syn` usage scan → call-site-scoped suggestions; embed in bro-desktop | M | 6 |
| APIDRIFT-10 | `api-drift` CLI: `check`, `note`, `affected`, `release-hint`, `--json` | S–M | 9 |
| APIDRIFT-11 | Fleet registry (`consumers.toml`) + foreman merge-gate hook | S | 9 |
| APIDRIFT-3 (existing) | applier / codemod | M | 5, 9 |

Order: 4 → 5 → 6 → 8 and 9 (the bro-acp / bro-desktop pair is the demo that
proves the idea) → 7 → 10 → 11. APIDRIFT-2 folds into 7.
