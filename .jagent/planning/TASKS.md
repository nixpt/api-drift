# Tasks — api-drift

## In progress

| ID | Task | Notes |
|----|------|-------|
| [APIDRIFT-8](./tickets/APIDRIFT-8-protocol-surface.md) | protocol surface — 8a api-drift-side (ProtocolSurface + schema_hash) | 8b bro-tui embed gated on bro-cli settling |

## Blocked

| ID | Task | Reason |
|----|------|--------|
| [APIDRIFT-9](./tickets/APIDRIFT-9-consumer-check.md) | consumer pins + check! + usage scan, bro-desktop | bro-cli actively changing (BRO-98 PR2 + BRO-113); wait for it to settle |

## Backlog

| ID | Task | Notes |
|----|------|-------|
| [APIDRIFT-10](./tickets/APIDRIFT-10-cli.md) | CLI: check/note done; affected/release-hint after 9/11 | check+note landed |
| [APIDRIFT-11](./tickets/APIDRIFT-11-registry-gate.md) | registry + merge-gate hook | after 9 |
| [APIDRIFT-12](./tickets/APIDRIFT-12-evorium-borrows.md) | Evorium borrows: guard done in core; relatedness+similarity remain | relatedness → 11 |
| [APIDRIFT-13](./tickets/APIDRIFT-13-checkstand-breadboard.md) | checkstand borrows + breadboard | schemars sig done in 8a; breadboard later |
| [APIDRIFT-3](./tickets/APIDRIFT-3-applier.md) | call-site applier / codemod | after 5, 9 |
| [APIDRIFT-15](./tickets/APIDRIFT-15-xray-nexus-inspect-json.md) | x-ray → nexus `inspect --json` consumer pair (M2 unblock, not in flux) | first; see docs/CONSUMERS.md |
| [APIDRIFT-16](./tickets/APIDRIFT-16-exosphere-capability-shell-surfaces.md) | exosphere CapabilityType/ShellRequest/CapsuleManifest surfaces; nexus `Request` mirror | after EXO-214/215 land |
| [APIDRIFT-17](./tickets/APIDRIFT-17-nakshatra-boot-toml-nak-env.md) | nakshatra boot.toml keys + NAK_* env (C producer); nexus recipe drift incident | motivating story |
| [APIDRIFT-18](./tickets/APIDRIFT-18-nexus-sdk-wire-gui-consumer.md) | nexus WireKey/IPC/keys/guests.d; nexus-gui consumer (bro-tui↔bro-desktop shape) | after NEXUS-056..059 merge |
| [APIDRIFT-19](./tickets/APIDRIFT-19-hawk-policy-rule-event-surfaces.md) | hawk PolicyRule + SecurityEvent; YAML usage scan | cheap non-Rust consumer |
| [APIDRIFT-20](./tickets/APIDRIFT-20-wsforge-profile-shell-surface.md) | wsforge profile keys vs script reads; shell usage scan | M3 |

## Done

| ID | Task | When |
|----|------|------|
| [APIDRIFT-1](./tickets/APIDRIFT-1-m0-scaffold.md) | M0 core pipeline scaffold | 2026-09-14 |
| [APIDRIFT-2](./tickets/APIDRIFT-2-rustdoc-producer.md) | rustdoc-JSON → ApiSnapshot producer | 2026-09-14 |
| [APIDRIFT-4](./tickets/APIDRIFT-4-core-correctness.md) | core correctness (review prerequisites) | 2026-09-14 |
| [APIDRIFT-5](./tickets/APIDRIFT-5-action-enum-serde.md) | action enum + attrs + serde + snapshot v1 | 2026-09-14 |
| [APIDRIFT-6](./tickets/APIDRIFT-6-surface-ledger.md) | Surface trait + ledger crate + dogfood | 2026-09-14 |
| [APIDRIFT-14](./tickets/APIDRIFT-14-catalog-surface.md) | data/catalog surface (ItemKind::Row + CatalogSurface) | 2026-09-14 |
| [APIDRIFT-8a](./tickets/APIDRIFT-8-protocol-surface.md) | protocol surface (ProtocolSurface + schema_hash) | 2026-09-15 |
| [APIDRIFT-7](./tickets/APIDRIFT-7-rust-public-surface.md) | rust-public surface + arniko ledger demo | 2026-09-15 |



- [ ] **opportunity** — Embedded contract-ledger design proposed (docs/DESIGN-embedded-ledger.md): ledger! test macro, committed snapshot+ledger.jsonl with migration notes, non-Rust surfaces (ACP/MCP/enum/CLI), consumer pins + call-site-scoped checks; first customer bro-acp/bro-desktop (bro-cli BRO-98). Prerequisite core fixes reproduced: old-side last-wins in diff, kind-checked rename detection, SignatureChanged→Breaking default.  _(unknown, 2026-09-14)_
