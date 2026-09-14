# APIDRIFT-3 — call-site applier

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | M |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

`Suggestion`s don't fix code by themselves; need a consumer that applies them.

## Goal

Call-site applier spec: `Suggestion` → patch (codemod / agent / human flow).

## Scope

- in: applier design, rename-call codemod prototype
- out: auto-landing without review (non-goal)

## Acceptance

- [ ] Applier spec written
- [ ] One codemod (rename-call) applies a real patch
- [ ] `cargo test` green (or the project's equivalent)

## Notes

Consumers: codemod script, agent prompt pack, human triage list.
Design-doc ordering: after APIDRIFT-5 + APIDRIFT-9 (needs the action enum
and the consumer usage scan to scope edits to real call sites).

