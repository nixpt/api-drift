//! Inherent members (`APIDRIFT-22`): the methods and associated consts of a
//! type, read from its `impls` list rather than from `paths`.
//!
//! rustdoc registers an inherent method in `paths` only when something links
//! to it, so a `paths`-only walk saw a handful of members per type. This pass
//! runs after re-export materialization and visits every struct/enum item
//! that carries an in-crate payload: the type's own entry and every local
//! alias of it (external aliases have no payload; the defining crate's own
//! snapshot owns those members).
//!
//! # Keys and signatures
//!
//! - Key: `<type path>::<member>`, the same spelling `paths` already used for
//!   linked-to methods, so those pre-existing entries are never rewritten.
//! - Same member name in several inherent impls of one type (`impl Foo<u8>`
//!   and `impl Foo<u16>`): every such member gets `<type path>::<member>@<self
//!   type>` (self type rendered from the impl's `for`, for example
//!   `Foo<u16>`). Rust forbids overlapping inherent impls defining one name,
//!   so distinct self types are enough; a residual tie gets `#2`, `#3`, ... in
//!   sorted-signature order.
//! - Signature: `{"impl":{"for":..,"generics":..},"<kind>":<member payload>}`
//!   normalized like every other payload, so an impl-level bound change is a
//!   signature change on each member. Members that `paths` already listed keep
//!   their legacy signature (the bare member payload): rewriting them would
//!   make the first run after upgrading report them all as changed.
//!
//! # Scope
//!
//! Public members of non-trait, non-synthetic, non-blanket impls. `Function`
//! stays the kind for methods (as it always was; `classify` treats `Function`
//! and `Method` identically), associated consts are `Const`, associated types
//! `Other`. A `#[doc(hidden)]` member is skipped, as rustdoc itself strips
//! such items from the index, so marking a method hidden reads as a removal.
//! Members under a local alias carry the `re_export` marker like the alias.

use super::{Doc, RawItem};
use crate::snapshot::{ItemKind, RE_EXPORT_ATTR};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

/// One public member of one inherent impl.
struct Member {
    name: String,
    kind: ItemKind,
    self_ty: String,
    payload: Value,
    attrs: Vec<String>,
}

impl Doc<'_> {
    /// Add every inherent member of every struct/enum in `by_path`.
    pub(super) fn add_inherent_members(&self, by_path: &mut BTreeMap<String, RawItem>) {
        let types: Vec<(String, Option<String>, bool, Value)> = by_path
            .values()
            .filter(|r| matches!(r.kind, ItemKind::Struct | ItemKind::Enum))
            .filter_map(|r| {
                let p = r.payload.as_ref()?;
                let is_alias = r.attrs.iter().any(|a| a == RE_EXPORT_ATTR);
                Some((r.path.clone(), r.target_path.clone(), is_alias, p.clone()))
            })
            .collect();
        for (type_path, target, is_alias, payload) in types {
            let members = self.members_of(&payload);
            for (suffix, m) in disambiguate(members) {
                let key = format!("{type_path}{suffix}");
                let mut attrs = m.attrs;
                if is_alias && !attrs.iter().any(|a| a == RE_EXPORT_ATTR) {
                    attrs.push(RE_EXPORT_ATTR.to_owned());
                }
                let legacy = by_path.contains_key(&key);
                if legacy && m.collides {
                    // The `paths` pass wrote the bare key for one of several
                    // same-named members; the disambiguated keys replace it.
                    by_path.remove(&key);
                } else if legacy {
                    continue;
                }
                // An alias of a type defined elsewhere in this crate points at
                // the canonical member (`re-export of ...`): the shape is
                // recorded once, where the type is defined, instead of once
                // per alias path.
                let (payload, target_path) = match (&target, is_alias) {
                    (Some(t), true) => (None, Some(format!("{t}{suffix}"))),
                    _ => (Some(m.payload), None),
                };
                by_path.insert(
                    key.clone(),
                    RawItem {
                        path: key,
                        kind: m.kind,
                        payload,
                        target_path,
                        attrs,
                    },
                );
            }
        }
    }

    fn members_of(&self, type_payload: &Value) -> Vec<Member> {
        let mut out = Vec::new();
        let impls = type_payload.get("impls").and_then(Value::as_array);
        for imp_id in impls.into_iter().flatten().filter_map(Value::as_u64) {
            let Some(imp) = self
                .node(&imp_id.to_string())
                .and_then(|n| n.get("inner"))
                .and_then(|i| i.get("impl"))
            else {
                continue;
            };
            let is_null = |k: &str| matches!(imp.get(k), None | Some(Value::Null));
            let is_inherent = is_null("trait")
                && is_null("blanket_impl")
                && imp.get("is_synthetic").and_then(Value::as_bool) != Some(true);
            if !is_inherent {
                continue;
            }
            let for_ty = imp.get("for").cloned().unwrap_or(Value::Null);
            let ctx = json!({"for": for_ty, "generics": imp.get("generics")});
            let self_ty = self.render_type(&for_ty);
            for id in imp
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(m) = id
                    .as_u64()
                    .and_then(|id| self.member(&id.to_string(), &ctx, &self_ty))
                {
                    out.push(m);
                }
            }
        }
        out
    }

    fn member(&self, id: &str, ctx: &Value, self_ty: &str) -> Option<Member> {
        let node = self.node(id)?;
        if !Self::is_public(node) || is_doc_hidden(node) {
            return None;
        }
        let name = node.get("name").and_then(Value::as_str)?.to_owned();
        let (key, payload) = node.get("inner")?.as_object()?.iter().next()?;
        let kind = match key.as_str() {
            "function" => ItemKind::Function,
            "assoc_const" | "constant" => ItemKind::Const,
            "assoc_type" => ItemKind::Other,
            _ => return None,
        };
        Some(Member {
            name,
            kind,
            self_ty: self_ty.to_owned(),
            payload: json!({"impl": ctx, key.as_str(): payload}),
            attrs: super::collect_attrs(node),
        })
    }

    /// Short, id-free rendering of an impl's self type, used only in keys:
    /// `Model<B>`, `Foo<u16>`, `&[T]`. Unknown shapes fall back to the
    /// normalized JSON, which is stable if ugly.
    fn render_type(&self, ty: &Value) -> String {
        let Some((tag, body)) = ty.as_object().and_then(|m| m.iter().next()) else {
            return "?".to_owned();
        };
        match (tag.as_str(), body) {
            ("primitive" | "generic", Value::String(s)) => s.clone(),
            ("resolved_path", p) => {
                let canonical = p
                    .get("id")
                    .and_then(Value::as_u64)
                    .and_then(|id| self.info.get(&id.to_string()))
                    .map(|i| i.dotted.as_str())
                    .or_else(|| p.get("path").and_then(Value::as_str))
                    .unwrap_or("?");
                let last = canonical.rsplit("::").next().unwrap_or(canonical);
                let args: Vec<String> = p
                    .pointer("/args/angle_bracketed/args")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|a| self.render_arg(a))
                    .collect();
                if args.is_empty() {
                    last.to_owned()
                } else {
                    format!("{last}<{}>", args.join(","))
                }
            }
            ("borrowed_ref", r) => {
                let m = if r.get("is_mutable").and_then(Value::as_bool) == Some(true) {
                    "mut "
                } else {
                    ""
                };
                format!(
                    "&{m}{}",
                    self.render_type(r.get("type").unwrap_or(&Value::Null))
                )
            }
            ("slice", t) => format!("[{}]", self.render_type(t)),
            ("tuple", Value::Array(ts)) => {
                let parts: Vec<String> = ts.iter().map(|t| self.render_type(t)).collect();
                format!("({})", parts.join(","))
            }
            _ => super::compact(&self.normalize(ty, &HashMap::new())),
        }
    }

    fn render_arg(&self, arg: &Value) -> String {
        if let Some(t) = arg.get("type") {
            self.render_type(t)
        } else if let Some(l) = arg.get("lifetime").and_then(Value::as_str) {
            l.to_owned()
        } else {
            super::compact(&self.normalize(arg, &HashMap::new()))
        }
    }
}

/// A member with its final key; `collides` marks names shared by several impls.
struct Keyed {
    kind: ItemKind,
    payload: Value,
    attrs: Vec<String>,
    collides: bool,
}

/// Assign each member its unique key suffix (`::name[@Self][#n]`, appended
/// to the type path; see the module docs).
fn disambiguate(members: Vec<Member>) -> BTreeMap<String, Keyed> {
    let mut by_name: BTreeMap<String, Vec<Member>> = BTreeMap::new();
    for m in members {
        by_name.entry(m.name.clone()).or_default().push(m);
    }
    let mut out = BTreeMap::new();
    for (name, group) in by_name {
        let collides = group.len() > 1;
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut group = group;
        // Deterministic tie order independent of rustdoc's impl numbering.
        group.sort_by_cached_key(|m| (m.self_ty.clone(), m.payload.to_string()));
        for m in group {
            let mut key = format!("::{name}");
            if collides {
                key = format!("{key}@{}", m.self_ty);
                let n = seen.entry(key.clone()).or_insert(0);
                *n += 1;
                if *n > 1 {
                    key = format!("{key}#{n}");
                }
            }
            out.insert(
                key,
                Keyed {
                    kind: m.kind,
                    payload: m.payload,
                    attrs: m.attrs,
                    collides,
                },
            );
        }
    }
    out
}

/// `#[doc(hidden)]` in either attribute encoding rustdoc has used.
fn is_doc_hidden(node: &Value) -> bool {
    node.get("attrs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|a| {
            let text = a.to_string();
            text.contains("doc") && text.contains("hidden")
        })
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value)] // fixture builders take JSON by value for readability
mod tests {
    use crate::classify::{classify_diff, BreakKind, Severity};
    use crate::diff::diff_snapshots;
    use crate::producer::snapshot_from_rustdoc_str;
    use crate::snapshot::{ApiSnapshot, ItemKind, RE_EXPORT_ATTR};
    use serde_json::{json, Value};

    fn func(name: &str, extra: Value) -> Value {
        let mut n = json!({
            "name": name, "visibility": "public",
            "inner": {"function": {"sig": {"inputs": [["self", {"generic": "Self"}]],
                                           "output": null}}}
        });
        if let (Some(o), Some(e)) = (n.as_object_mut(), extra.as_object()) {
            o.extend(e.clone());
        }
        n
    }

    fn impl_node(items: &[u64], for_ty: Value, generics: Value, extra: Value) -> Value {
        let mut imp = json!({
            "trait": null, "blanket_impl": null, "is_synthetic": false,
            "for": for_ty, "generics": generics, "items": items
        });
        if let (Some(o), Some(e)) = (imp.as_object_mut(), extra.as_object()) {
            o.extend(e.clone());
        }
        json!({"visibility": "default", "inner": {"impl": imp}})
    }

    fn ty(name: &str, id: u64, args: Value) -> Value {
        json!({"resolved_path": {"path": name, "id": id, "args": args}})
    }

    fn no_generics() -> Value {
        json!({"params": [], "where_predicates": []})
    }

    fn bound(b: &str) -> Value {
        json!({"params": [{"name": "B", "kind": {"type": {"bounds": [
            {"trait_bound": {"trait": {"path": b, "id": if b.ends_with("Other") { 901 } else { 900 },
                       "args": null}}}]}}}],
            "where_predicates": []})
    }

    /// `struct Foo` (id `s`) with one inherent impl holding `m`, a method
    /// that no `paths` entry names (nothing links to it).
    fn simple(s: u64, imp: u64, m: u64, generics: Value) -> Value {
        json!({
            "index": {
                "1": {"visibility": "public", "inner": {"module": {"items": [s]}}},
                s.to_string(): {"visibility": "public", "name": "Foo",
                    "inner": {"struct": {"kind": "unit", "impls": [imp]}}},
                imp.to_string(): impl_node(&[m], ty("Foo", s, Value::Null), generics, json!({})),
                m.to_string(): func("run", json!({}))
            },
            "paths": {
                "1": {"crate_id": 0, "path": ["demo"], "kind": "module"},
                s.to_string(): {"crate_id": 0, "path": ["demo", "Foo"], "kind": "struct"},
                "900": {"crate_id": 3, "path": ["ext", "Backend"], "kind": "trait"},
                "901": {"crate_id": 3, "path": ["ext", "Other"], "kind": "trait"}
            }
        })
    }

    fn snap(v: &Value) -> ApiSnapshot {
        snapshot_from_rustdoc_str(&v.to_string(), "v").unwrap()
    }

    #[test]
    fn method_without_a_doc_link_is_tracked() {
        let s = snap(&simple(2, 3, 4, no_generics()));
        let m = s.get("demo::Foo::run").expect("unlinked method surfaced");
        assert_eq!(m.kind, ItemKind::Function);
        assert!(
            m.sig.contains(r#""impl""#),
            "sig carries impl context: {}",
            m.sig
        );
    }

    #[test]
    fn removing_or_changing_a_method_is_breaking_adding_is_not() {
        let old = snap(&simple(2, 3, 4, no_generics()));
        let mut gone = simple(2, 3, 4, no_generics());
        gone["index"]["3"]["inner"]["impl"]["items"] = json!([]);
        let d = diff_snapshots(&old, &snap(&gone));
        let b = classify_diff(&d);
        assert_eq!(b.len(), 1);
        assert_eq!(
            (b[0].kind, b[0].severity),
            (BreakKind::Removed, Severity::Breaking)
        );
        // The reverse direction is an addition: not breaking.
        let d = diff_snapshots(&snap(&gone), &old);
        assert_eq!(d.added.len(), 1);
        assert!(classify_diff(&d)
            .iter()
            .all(|b| b.severity != Severity::Breaking));
        // Signature move.
        let mut changed = simple(2, 3, 4, no_generics());
        changed["index"]["4"]["inner"]["function"]["sig"]["output"] = json!({"primitive": "u8"});
        let c = classify_diff(&diff_snapshots(&old, &snap(&changed)));
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, BreakKind::SignatureChanged);
        assert_eq!(c[0].severity, Severity::Breaking);
    }

    #[test]
    fn impl_level_bound_change_is_a_signature_change() {
        let a = snap(&simple(2, 3, 4, bound("ext::Backend")));
        let b = snap(&simple(2, 3, 4, bound("ext::Other")));
        let d = diff_snapshots(&a, &b);
        assert_eq!(d.changed.len(), 1, "{d:?}");
        assert_eq!(d.changed[0].path, "demo::Foo::run");
    }

    #[test]
    fn ids_are_not_part_of_the_snapshot() {
        let a = snap(&simple(2, 3, 4, bound("ext::Backend")));
        let b = snap(&simple(20, 31, 47, bound("ext::Backend")));
        assert!(diff_snapshots(&a, &b).is_empty());
        let key = |s: &ApiSnapshot| {
            let mut v: Vec<_> = s
                .items
                .iter()
                .map(|i| (i.path.clone(), i.sig.clone()))
                .collect();
            v.sort();
            v
        };
        assert_eq!(key(&a), key(&b), "byte-identical across renumbering");
    }

    /// `impl Foo<u8> { fn new }` and `impl Foo<u16> { fn new }`.
    fn two_impls(order: [u64; 2]) -> Value {
        let args = |t: &str| json!({"angle_bracketed": {"args": [{"type": {"primitive": t}}]}});
        let mut doc = simple(2, 3, 4, no_generics());
        doc["index"]["2"]["inner"]["struct"]["impls"] = json!(order);
        doc["index"]["3"] = impl_node(&[4], ty("Foo", 2, args("u8")), no_generics(), json!({}));
        doc["index"]["5"] = impl_node(&[6], ty("Foo", 2, args("u16")), no_generics(), json!({}));
        doc["index"]["4"] = func("new", json!({}));
        doc["index"]["6"] = func("new", json!({"id": 6}));
        doc["index"]["6"]["inner"]["function"]["sig"]["output"] = json!({"primitive": "u16"});
        doc
    }

    #[test]
    fn same_name_in_two_impls_gets_unique_deterministic_keys() {
        let a = snap(&two_impls([3, 5]));
        assert!(a.get("demo::Foo::new@Foo<u8>").is_some());
        assert!(a.get("demo::Foo::new@Foo<u16>").is_some());
        assert!(
            a.get("demo::Foo::new").is_none(),
            "bare key would be ambiguous"
        );
        // Independent of the order rustdoc lists the impls in.
        assert!(diff_snapshots(&a, &snap(&two_impls([5, 3]))).is_empty());
        // Changing one instantiation moves only that key.
        let mut changed = two_impls([3, 5]);
        changed["index"]["6"]["inner"]["function"]["sig"]["output"] = json!({"primitive": "u32"});
        let d = diff_snapshots(&a, &snap(&changed));
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].path, "demo::Foo::new@Foo<u16>");
    }

    #[test]
    fn members_already_listed_in_paths_keep_their_old_spelling() {
        let mut doc = simple(2, 3, 4, bound("ext::Backend"));
        doc["paths"]["4"] = json!({"crate_id": 0, "path": ["demo", "Foo", "run"],
                                   "kind": "function"});
        let s = snap(&doc);
        let m = s.get("demo::Foo::run").unwrap();
        assert!(
            !m.sig.contains(r#""impl""#),
            "legacy entry untouched: {}",
            m.sig
        );
        assert_eq!(
            s.items.iter().filter(|i| i.path.ends_with("::run")).count(),
            1
        );
    }

    #[test]
    fn local_alias_carries_members_as_a_pointer_to_the_canonical_path() {
        let mut doc = simple(2, 3, 4, no_generics());
        doc["index"]["1"]["inner"]["module"]["items"] = json!([2, 7]);
        doc["index"]["7"] = json!({"visibility": "public", "inner": {"use": {
            "source": "demo::Foo", "name": "Bar", "id": 2, "is_glob": false}}});
        let s = snap(&doc);
        let canon = s.get("demo::Foo::run").unwrap();
        assert!(!canon.has_attr(RE_EXPORT_ATTR));
        let alias = s
            .get("demo::Bar::run")
            .expect("method under the alias path");
        assert!(alias.has_attr(RE_EXPORT_ATTR));
        assert_eq!(alias.sig, "re-export of demo::Foo::run");
        // Dropping the re-export removes the alias member too.
        let d = diff_snapshots(&s, &snap(&simple(2, 3, 4, no_generics())));
        assert!(d.removed.iter().any(|i| i.path == "demo::Bar::run"));
    }

    #[test]
    fn doc_hidden_members_disappear() {
        let mut doc = simple(2, 3, 4, no_generics());
        let before = snap(&doc);
        doc["index"]["4"]["attrs"] = json!([{"other": "#[doc(hidden)]"}]);
        let d = diff_snapshots(&before, &snap(&doc));
        assert_eq!(d.removed.len(), 1, "hiding a method reads as a removal");
        doc["index"]["4"]["attrs"] = json!(["#[doc(hidden)]"]);
        assert!(snap(&doc).get("demo::Foo::run").is_none());
        // Unrelated attrs do not hide.
        doc["index"]["4"]["attrs"] = json!([{"other": "#[inline]"}]);
        assert!(snap(&doc).get("demo::Foo::run").is_some());
    }

    #[test]
    fn associated_consts_and_non_public_or_trait_members_are_handled() {
        let mut doc = simple(2, 3, 4, no_generics());
        doc["index"]["3"]["inner"]["impl"]["items"] = json!([4, 5, 6]);
        doc["index"]["5"] = json!({"name": "LIMIT", "visibility": "public",
            "inner": {"assoc_const": {"type": {"primitive": "u8"}, "value": "3"}}});
        doc["index"]["6"] = func("secret", json!({"visibility": "crate"}));
        // A trait impl, a blanket impl and an auto-trait impl contribute nothing.
        doc["index"]["8"] = impl_node(
            &[9],
            ty("Foo", 2, Value::Null),
            no_generics(),
            json!({"trait": {"path": "Display", "id": 902, "args": null}}),
        );
        doc["index"]["10"] = impl_node(
            &[9],
            ty("Foo", 2, Value::Null),
            no_generics(),
            json!({"blanket_impl": {"generic": "T"}}),
        );
        doc["index"]["11"] = impl_node(
            &[9],
            ty("Foo", 2, Value::Null),
            no_generics(),
            json!({"is_synthetic": true}),
        );
        doc["index"]["9"] = func("fmt", json!({}));
        doc["index"]["2"]["inner"]["struct"]["impls"] = json!([3, 8, 10, 11]);
        let s = snap(&doc);
        assert_eq!(s.get("demo::Foo::LIMIT").unwrap().kind, ItemKind::Const);
        assert!(s.get("demo::Foo::run").is_some());
        assert!(s.get("demo::Foo::secret").is_none());
        assert!(s.get("demo::Foo::fmt").is_none());
    }

    #[test]
    fn external_alias_without_a_payload_has_no_members() {
        let doc = json!({
            "index": {
                "1": {"visibility": "public", "inner": {"module": {"items": [2]}}},
                "2": {"visibility": "public", "inner": {"use": {
                    "source": "ext::T", "name": "T", "id": 10, "is_glob": false}}}
            },
            "paths": {
                "1": {"crate_id": 0, "path": ["demo"], "kind": "module"},
                "10": {"crate_id": 7, "path": ["ext", "T"], "kind": "struct"}
            }
        });
        let s = snap(&doc);
        assert_eq!(s.len(), 1, "only the re-export marker itself");
    }

    #[test]
    fn enums_get_members_too() {
        let mut doc = simple(2, 3, 4, no_generics());
        let st = doc["index"]["2"]["inner"]["struct"].take();
        doc["index"]["2"]["inner"] = json!({"enum": st});
        doc["paths"]["2"]["kind"] = json!("enum");
        assert!(snap(&doc).get("demo::Foo::run").is_some());
    }
}
