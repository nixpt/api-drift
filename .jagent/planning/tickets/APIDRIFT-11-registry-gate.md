# APIDRIFT-11 — fleet registry (consumers.toml) + foreman merge-gate hook

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M2 |
| **Size** | S |
| **Owner** | unassigned |
| **Created** | 2026-09-14 |
| **Updated** | 2026-09-14 |

## Problem

Nothing maps an upstream ledger change to its affected downstream repos at
merge time.

## Goal

`consumers.toml` registry (self-maintained via `api-drift check`) + foreman
merge-gate hook printing affected repos and call-site counts for PRs
touching a ledgered surface — the post-merge semantic-conflict class caught
pre-merge.

## Scope

- in: registry format, self-registration, gate hook
- out: —

## Acceptance

- [ ] Gate prints affected downstream + call-site counts on a ledgered PR
- [ ] Registry stays advisory (never blocks on stale data)

## Notes

Depends on APIDRIFT-9.
