# APIDRIFT-20 — wsforge: `profiles/*.conf` variable set vs `bin/*` reads — a shell catalog surface

| Field | Value |
|-------|--------|
| **Status** | backlog |
| **Milestone** | M3 |
| **Size** | S |
| **Owner** | unassigned |
| **Created** | 2026-09-16 |
| **Updated** | 2026-09-16 |

## Problem

`profiles/nixps.conf` assigns 32 variables; `bin/wsforge-configure` alone reads 36 distinct
`$VARS`. Nobody measures the difference, and WSF-2 adds `NEXUS_VT`, `NEXUS_CONSOLE_ENTRY`,
`NEXUS_GUI_SESSION`. A profile is a data contract between a declarative file and three
scripts — the same shape as checkstand's catalog (APIDRIFT-14) with a shell consumer.

## Goal

`catalog_surface!` over profile keys (sig = type/default/required) in a tiny test crate under
`wsforge/crates/` (one already exists), ledger, and a shell `$VAR` reference extractor for
`check!` so a script reading an undeclared key, or a profile declaring an unread key, fails
`cargo test`.

## Scope

- in: surface, ledger, shell extractor; WSF-2 keys as the first entry.
- out: the provisioning work (WSF-2).

## Acceptance

- [ ] The 36-vs-32 gap is enumerated by the first `check!` run and each item is either
      declared, removed, or recorded as intentional.
