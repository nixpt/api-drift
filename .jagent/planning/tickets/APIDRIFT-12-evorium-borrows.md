# APIDRIFT-12 — borrow from Evorium: antibodies, autoimmune guard, relatedness

| Field | Value |
|-------|--------|
| **Status** | in_progress — antibodies (6) + guard done; relatedness + similarity remain |
| **Milestone** | M2 |
| **Size** | S–M (spread across 6/9/11) |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-15 |

## Problem

The design review predates `projects/evorium`'s concrete mechanisms. Several
of them map one-to-one onto the embedded ledger and improve 6/9/11 — cheaper
than rediscovering them.

## Goal

Adopt the mappings in `docs/DESIGN-embedded-ledger.md` § "Prior art within
the fleet: Evorium":

1. **Antibody ledger notes** (→ APIDRIFT-6/9): the migration note is a shared
   antibody; add `corroboration: u32` rising as more consumers confirm the
   same break; `check!` distributes immunity pre-merge.
2. **Autoimmune guard** (→ ledger auto-applier): `Suggestion` gains
   `confidence: f64`; the applier consults a guard so low-confidence actions
   never exceed a rung (our APIDRIFT-4 conservatism, formalized — mirrors
   `immune::autoimmune_guard`).
3. **Relatedness ranking** (→ APIDRIFT-11 gate): rank affected downstream
   repos by ancestor-set Jaccard (`genome::TreeOfCode::relatives` pattern),
   not by list order.
4. **Rename similarity metric** (→ core, optional): evaluate cosine-over-
   counts (`genome::Anatomy::structural_similarity`) against the current
   edit-distance ≤ 8 weak signal; adopt only if it demonstrably improves
   same-kind rename recall.
5. **Signed ledger notes** (→ 6, deferred): `did:key`-signed notes with
   corroboration + `--min-trust` for auto-apply; start unsigned + advisory
   (immune's own caveat: unsigned antibodies are advisory).
6. **Ledger workflow naming** (→ 6): diff (plan) → verify → append (commit);
   corrections append, never rewrite (append-only jsonl already enforces).

## Scope

- in: the six mappings above, landed inside 6/9/11 rather than as new deps
- out: depending on evorium-* crates as path/git deps (borrow patterns +
  vocabulary, not code — evorium is OCPL-1.1 and OpenKO-governed; keep
  api-drift MIT OR Apache-2.0 and decoupled)

## Acceptance

- [x] Ledger entry format (APIDRIFT-6) includes note + `corroboration`
- [x] Auto-applier consults a confidence guard — `Suggestion.confidence: f64`
      + `autoimmune_guard` landed in core (15)
- [ ] Gate output ranks repos by relatedness (landed with 11)
- [ ] Similarity-metric comparison documented (adopt or reject with reason)

## Notes

Provenance: `projects/evorium` crates `evorium-immune` (`Antibody`,
`ImmuneMemory`, `autoimmune_guard`), `evorium-genome` (`TreeOfCode`,
`Anatomy::structural_similarity`), `evorium-forge` (`PatchArtifact`,
`Channel`, `ManifestDelta`), `evorium-canopy` (transition cycle). License
note: evorium is OCPL-1.1 — pattern-borrow only, no code lift.
