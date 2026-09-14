# `.jagent/planning/` — project execution board

**What this is:** roadmap, tasks, tickets, issues, PRs, milestones — the
*what / when / who* of api-drift delivery.

**What this is not:** architectural *why*. That stays in **`.dejavue/`**
(canonical).

| Need | Go to |
|------|--------|
| Why we chose X | `.dejavue/` → `dejavue context` / `decision` / `recall` |
| What's next to build | `ROADMAP.md`, `TASKS.md`, `tickets/` |
| Current delivery state | `STATE.md` |
| Project identity | `.jagent/PROJECT.md` |
| Active todos | `.jagent/TODO.md` — `joker_list_todos` |
| Bug / debt | `issues/` |
| Open / landed PRs | `prs/` |
| Session work log | `WORK_LOG.md` |

## Layout

```text
.jagent/
├── TODO.md              # joker_list_todos / joker_complete_todo
├── PROJECT.md           # One-page project identity + key decisions
└── planning/
    ├── README.md        # this file
    ├── ROADMAP.md       # phases & milestones — sequence
    ├── STATE.md         # delivery snapshot — update often
    ├── TASKS.md         # kanban-style backlog
    ├── WORK_LOG.md      # append-only work notes
    ├── tickets/         # PROJECT-NNN work items
    ├── milestones/      # one file per milestone
    ├── issues/          # bugs / debt / risks
    └── prs/             # PR tracking (local / GitHub)
```

## Ticket IDs

`APIDRIFT-NN` — next free number in `tickets/`.
Branch convention (when remote exists): `agent/<name>/APIDRIFT-NN`.

## Workflow (agents)

1. `dejavue context` — architectural memory
2. Read `.jagent/TODO.md` + `planning/STATE.md` + `TASKS.md`
3. Claim or open a ticket under `tickets/` (`APIDRIFT-NN`)
4. Implement; append a block to `WORK_LOG.md`
5. If architectural: `dejavue decision` (+ invariant/trap if needed)
6. Update ticket status + `STATE.md` + milestone file when done
7. Session end: `dejavue state` + `dejavue handoff`

**Current ticket range:** APIDRIFT-1…3.
