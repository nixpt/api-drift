# Decisions


## 2026-09-14T18:01:00-05:00 — [ADOPTED] [ARCHITECTURAL] Core is pure std with zero default dependencies and no I/O; producers and serialization live behind cargo features (producer, serde) or in sibling crates.

Reason:
Leaf posture (like nixpt-common / pid-event-policy): the policy layer must be embeddable anywhere, including as a dev-dependency test in other crates, without dragging serde or I/O into consumers that only want diff/classify/suggest. Rejected: a single crate with serde always-on.

Author type: agent


## 2026-09-14T18:01:01-05:00 — [ADOPTED] [ARCHITECTURAL] classify errs toward Breaking: function/method signature changes are Breaking unless the widening allowlist (&str -> impl Into<String>, &T -> impl AsRef<T>, ...) proves source compatibility; added enum variants and struct fields are Warning unless the parent is non_exhaustive or the item is deprecated.

Reason:
Rust has no default arguments, so almost every signature change breaks call sites; the original scaffold labelled them Warning and a review on 2026-09-14 showed a new required arg would have been under-labelled. Consumers (codemods, agents) will act on these labels, so false 'compatible' is the expensive error. APIDRIFT-4.

Author type: agent


## 2026-09-14T18:01:01-05:00 — [ADOPTED] [ARCHITECTURAL] Rename detection requires same parent module AND same ItemKind AND a signal: identical signature modulo the leaf name gives an auto-appliable RenameCall; a small edit distance gives a review-only ReviewRename. ClassifiedBreak carries item_kind so suggest can check it.

Reason:
The scaffold folded any removed+added pair under one parent into an auto rename, which auto-applied a removed field + added method as a call-site rewrite (reproduced 2026-09-14). auto_appliable must mean a script can apply it without judgment. APIDRIFT-4.

Author type: agent


## 2026-09-14T18:01:02-05:00 — [ADOPTED] [ARCHITECTURAL] Suggestion.action is a structured SuggestionAction enum (RenameCall, ReviewRename, RemoveItem, ReviewSignature, ReviewKind, AddMatchArm, ReviewField, NoOp) with typed fields; detail stays as the rendered human string.

Reason:
Consumers are codemods that must match on the action; stringly verbs invited drift between README promises and code. APIDRIFT-5. Rejected: keep strings and document them.

Author type: agent


## 2026-09-14T18:01:02-05:00 — [ADOPTED] [ARCHITECTURAL] Snapshot file format v1: {format: api-drift/snapshot/v1, surface, version, items sorted by path, content_hash: sha256}. Parse fails closed on unknown format or hash mismatch; any field change is a new format string.

Reason:
The file is the committed contract that ledger entries and consumer pins reference by hash; a tampered or drifted contract must fail, not warn. APIDRIFT-5.

Author type: agent


## 2026-09-14T18:01:03-05:00 — [ADOPTED] [ARCHITECTURAL] Direction: api-drift becomes an embedded contract ledger — a dev-dependency test (ledger!) per project keeping committed snapshots + an append-only ledger.jsonl with migration notes, surfaces beyond pub items (protocol methods, enum variants, CLI flags), and consumer pins that intersect breaks with call sites. First customer: bro-acp (bro_tui::backend::BackendEvent) with bro-desktop as consumer.

Reason:
Captain's idea 2026-09-14: the value is not diffing two files by hand but every project recording its own contract so downstream breaks are inferred before merge. Design in docs/DESIGN-embedded-ledger.md; tickets APIDRIFT-6..11. Rejected: staying a point-at-two-snapshots library.

Author type: agent


## 2026-09-14T18:01:03-05:00 — [ADOPTED] [ARCHITECTURAL] Release posture: MIT OR Apache-2.0 with both LICENSE files, crates.io name api-drift (free as of 2026-09-14), rust-version 1.74 (cargo [lints] table is the binding constraint), Cargo.lock kept at format v3, package exclude drops .jagent/.dejavue/scripts/docs/CLAUDE.md/STATE.md, CI = fmt + clippy + test + rustdoc -D warnings + MSRV job + publish dry run.

Reason:
Captain wants newer projects public and release-ready; first CI run failed because lockfile v4 needs cargo 1.78, so v3 keeps the MSRV honest instead of raising it. Rejected: rust-version 1.78 just to keep lock v4.

Author type: agent

