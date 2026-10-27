# APIDRIFT-22 — Producer: track inherent methods and associated consts

| Field | Value |
|-------|-------|
| **Status** | in review |
| **Milestone** | M1 (producer hardening) |
| **Size** | M |
| **Owner** | vega |
| **Created** | 2026-10-27 |
| **Updated** | 2026-10-27 |
| **Driver** | zorro ZORRA-3-19 (the gate could not see `Model::generate_stream`) |

## Problem

The producer built its item set from rustdoc JSON's `paths` map only. rustdoc
registers an inherent method in `paths` only when something references it (an
intra-doc link), and the producer never walked a type's `impls`. On zorro
(nightly-2026-08-28) 1,025 public inherent fns/consts exist on crate-local
public types; the committed snapshots listed about 150 of them, and
`zorro_engine::engine::Model` showed 4 of its 18 methods.

## What changed

New pass `Doc::add_inherent_members` (`producer_members.rs`), run after
re-export materialization:

- Walks `inner.{struct,enum}.impls` of every emitted struct/enum item (direct
  path and local aliases). Only NON-trait, non-synthetic, non-blanket impls;
  only PUBLIC members that are not `#[doc(hidden)]`.
- Key `<type path>::<member>`; same name in several inherent impls becomes
  `<type path>::<member>@<self type>` (`Foo::new@Foo<u8>`), `#n` for a residual
  tie. Independent of rustdoc impl order and ids.
- Sig = `{"impl":{"for","generics"},"<kind>":payload}` normalized like the rest,
  so an impl-level bound change is a signature change of each member.
- Members `paths` already listed keep their legacy bare spelling (additions-only
  rollout); they lack the impl context until a deliberate re-bless.
- A local alias of a type (`pub use inner::Foo as Bar`) gets `Bar::m` with
  `re_export` and sig `re-export of <canonical>::m` (shape recorded once).
  External aliases have no payload: the defining crate's snapshot owns them.
- `Kind`: unchanged. Methods stay `ItemKind::Function` (classify treats it like
  `Method`; switching kinds would change legacy entries), assoc consts `Const`,
  assoc types `Other`. Removal and signature change = Breaking, addition =
  compatible, same as free functions.
- `#[doc(hidden)]` members are skipped (rustdoc also strips them from the index),
  so hiding a method is reported as a removal.

## Known limits

- Default cfg only (the consumer's rustdoc invocation decides).
- Unions are not emitted as items by the producer, so their members are not
  tracked either.
- The ~150 legacy entries keep sigs without impl context.

## Verification

Synthetic fixtures in `producer_members.rs` (unlinked method, two impls same
name, alias, doc(hidden), assoc const, impl bound change, id renumbering,
trait/blanket/synthetic/private impls, enums) plus a real-JSON differential on
zorro's 34 crates: 0 removed, 0 changed on existing entries.
