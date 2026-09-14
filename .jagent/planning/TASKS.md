# Tasks — api-drift

## In progress

_None — APIDRIFT-5 just landed._

## Ready

- [APIDRIFT-6](./tickets/APIDRIFT-6-surface-ledger.md) — Surface trait + ledger crate + dogfood | M2 |

## Backlog

| ID | Task | Notes |
|----|------|-------|
| [APIDRIFT-8](./tickets/APIDRIFT-8-protocol-surface.md) | protocol surface + bro-tui embed (BRO-98) | after 6, ∥ 9 |
| [APIDRIFT-9](./tickets/APIDRIFT-9-consumer-check.md) | consumer pins + check! + usage scan, bro-desktop | after 6, ∥ 8 |
| [APIDRIFT-7](./tickets/APIDRIFT-7-rust-public-surface.md) | rust-public surface, arniko ledger demo | after 6 |
| [APIDRIFT-10](./tickets/APIDRIFT-10-cli.md) | CLI: check/note/affected/release-hint | after 9 |
| [APIDRIFT-11](./tickets/APIDRIFT-11-registry-gate.md) | registry + merge-gate hook | after 9 |
| [APIDRIFT-12](./tickets/APIDRIFT-12-evorium-borrows.md) | Evorium borrows: antibodies, guard, relatedness | woven into 6/9/11 |
| [APIDRIFT-3](./tickets/APIDRIFT-3-applier.md) | call-site applier / codemod | after 5, 9 |

## Done

| ID | Task | When |
|----|------|------|
| [APIDRIFT-1](./tickets/APIDRIFT-1-m0-scaffold.md) | M0 core pipeline scaffold | 2026-09-14 |
| [APIDRIFT-2](./tickets/APIDRIFT-2-rustdoc-producer.md) | rustdoc-JSON → ApiSnapshot producer | 2026-09-14 |
| [APIDRIFT-4](./tickets/APIDRIFT-4-core-correctness.md) | core correctness (review prerequisites) | 2026-09-14 |
| [APIDRIFT-5](./tickets/APIDRIFT-5-action-enum-serde.md) | action enum + attrs + serde + snapshot v1 | 2026-09-14 |


- [ ] **opportunity** — Embedded contract-ledger design proposed (docs/DESIGN-embedded-ledger.md): ledger! test macro, committed snapshot+ledger.jsonl with migration notes, non-Rust surfaces (ACP/MCP/enum/CLI), consumer pins + call-site-scoped checks; first customer bro-acp/bro-desktop (bro-cli BRO-98). Prerequisite core fixes reproduced: old-side last-wins in diff, kind-checked rename detection, SignatureChanged→Breaking default.  _(unknown, 2026-09-14)_
