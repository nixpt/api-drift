//! rustdoc-JSON → [`ApiSnapshot`] producer.
//!
//! Requires the `producer` cargo feature (`serde_json`). Input is a rustdoc
//! JSON file as emitted by:
//!
//! ```bash
//! cargo +nightly rustdoc -- -Z unstable-options --output-format json
//! ```
//!
//! # What becomes an item
//!
//! - **Direct items**: in-crate (`crate_id == 0`) public items listed in the
//!   `paths` table with a known `inner` kind.
//! - **Re-exported items** (`APIDRIFT-21`): public `use` items are
//!   *materialized* at their reachable local path. A module alias
//!   (`pub use zorro_io as io;`) expands every public item under the target
//!   path to `zorro::io::…`; a named re-export emits one alias; a glob
//!   expands the target's public contents into the parent module. Chains
//!   (`pub use a::X` pointing at another local `use`) are followed, and
//!   globs of private local modules are walked through the `index` module
//!   tree, which `paths` cannot express. Aliased items carry the
//!   `re_export` attribute marker and a stable signature: the target's own
//!   normalized payload for in-crate targets, or `re-export of <path>` for
//!   external ones (the leaf crate owns that shape and its own ledger).
//!
//! # Signature stability
//!
//! Signature strings are the compact `inner` JSON payload with rustdoc item
//! ids normalized (`APIDRIFT-21`): every `id` field and every id in an
//! `impls` / `fields` / `variants` list becomes a path or a local name, and
//! impl lists are sorted. Ids renumber on every rebuild and across a
//! decomposition even when the shape is identical, so raw ids made
//! `SignatureChanged` noise.
//!
//! # Limitations (documented, not silent)
//!
//! - Methods surface as `ItemKind::Function` (rustdoc lists them as
//!   `function` nodes; no Method split yet).
//! - `sig` is compact JSON, not `pub fn …` rendering — machine-diffable,
//!   human-ugly.
//! - Unknown `inner` shapes are skipped, never error (newer rustdoc degrades
//!   to fewer items, not failure). Validated against format v58–v61.
//! - External re-exports only have the target's *path*, not its signature
//!   (rustdoc JSON does not inline dependency items); the signature is
//!   therefore the stable `re-export of …` marker.

use crate::snapshot::{ApiSnapshot, Item, ItemKind, RE_EXPORT_ATTR};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[cfg(feature = "producer")]
use serde_json::{Map, Value};

/// What went wrong reading a rustdoc JSON document.
#[derive(Debug)]
pub enum ProducerError {
    /// File could not be read.
    Io(std::io::Error),
    /// File is not valid JSON.
    Json(serde_json::Error),
    /// JSON parses but has no usable `index`/`paths` tables.
    BadShape(String),
}

impl fmt::Display for ProducerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "read rustdoc JSON: {e}"),
            Self::Json(e) => write!(f, "parse rustdoc JSON: {e}"),
            Self::BadShape(m) => write!(f, "unexpected rustdoc JSON shape: {m}"),
        }
    }
}

impl From<std::io::Error> for ProducerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ProducerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Read a rustdoc JSON file and build an [`ApiSnapshot`] labelled `version`.
pub fn snapshot_from_rustdoc_json(
    path: &std::path::Path,
    version: &str,
) -> Result<ApiSnapshot, ProducerError> {
    let text = std::fs::read_to_string(path)?;
    snapshot_from_rustdoc_str(&text, version)
}

/// Same as [`snapshot_from_rustdoc_json`] but from an in-memory document.
pub fn snapshot_from_rustdoc_str(doc: &str, version: &str) -> Result<ApiSnapshot, ProducerError> {
    let root: Value = serde_json::from_str(doc)?;
    parse_doc(&root, version)
}

/// One `paths` entry: canonical dotted path + kind + defining crate.
#[derive(Clone, Debug)]
struct PathInfo {
    dotted: String,
    kind: String,
    crate_id: u64,
}

/// Parsed view over one rustdoc JSON document.
struct Doc<'a> {
    index: &'a Map<String, Value>,
    root: String,
    info: HashMap<String, PathInfo>,
    /// child item id → direct parent module id (local items only).
    parent: HashMap<String, String>,
    /// public module id → its reachable dotted path.
    module_path: HashMap<String, String>,
}

impl<'a> Doc<'a> {
    fn new(index: &'a Map<String, Value>, paths: &'a Map<String, Value>, root_id: Option<&str>) -> Self {
        let mut info = HashMap::new();
        for (id, p) in paths {
            let dotted = p
                .get("path")
                .and_then(Value::as_array)
                .map(|segs| {
                    segs.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("::")
                })
                .unwrap_or_default();
            if dotted.is_empty() {
                continue;
            }
            info.insert(
                id.clone(),
                PathInfo {
                    dotted,
                    kind: p
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("other")
                        .to_owned(),
                    crate_id: p.get("crate_id").and_then(Value::as_u64).unwrap_or(u64::MAX),
                },
            );
        }

        let mut parent = HashMap::new();
        let mut module_path = HashMap::new();
        for (id, node) in index {
            if let Some(items) = node
                .get("inner")
                .and_then(|i| i.get("module"))
                .and_then(|m| m.get("items"))
                .and_then(Value::as_array)
            {
                for child in items.iter().filter_map(Value::as_u64) {
                    parent.insert(child.to_string(), id.clone());
                }
                if let Some(p) = info.get(id) {
                    module_path.insert(id.clone(), p.dotted.clone());
                }
            }
        }

        // The root module's `paths` entry may be absent in some formats; its
        // node carries the crate name. Synthetic fixtures may omit `root`
        // entirely — fall back to the parentless crate-local module.
        let root_id = root_id.map(str::to_owned).or_else(|| {
            index
                .iter()
                .find(|(id, node)| {
                    node.get("crate_id").and_then(Value::as_u64) == Some(0)
                        && node.get("inner").and_then(|i| i.get("module")).is_some()
                        && !parent.contains_key(*id)
                })
                .map(|(id, _)| id.clone())
        });
        let root = root_id
            .as_ref()
            .and_then(|id| info.get(id))
            .map(|p| p.dotted.clone())
            .or_else(|| {
                root_id
                    .as_ref()
                    .and_then(|id| index.get(id))
                    .and_then(|n| n.get("name"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "crate".to_owned());
        if let Some(id) = &root_id {
            module_path.insert(id.clone(), root.clone());
        }

        Self { index, root, info, parent, module_path }
    }

    fn node(&self, id: &str) -> Option<&'a Value> {
        self.index.get(id)
    }

    fn use_parts(&self, id: &str) -> Option<UseParts<'_>> {
        let node = self.node(id)?;
        let u = node.get("inner")?.get("use")?;
        Some(UseParts {
            source: u.get("source").and_then(Value::as_str).unwrap_or_default(),
            name: u.get("name").and_then(Value::as_str),
            target: u.get("id").and_then(Value::as_u64),
            is_glob: u.get("is_glob").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    /// Resolve a `use` chain to the first id with a canonical `paths` entry
    /// (or to a local module/item when `paths` has none).
    fn resolve_target(&self, id: u64) -> Option<String> {
        let mut cur = id.to_string();
        for _ in 0..16 {
            if self.info.contains_key(&cur) {
                return Some(cur);
            }
            let node = self.node(&cur)?;
            if let Some(t) = node
                .get("inner")
                .and_then(|i| i.get("use"))
                .and_then(|u| u.get("id"))
                .and_then(Value::as_u64)
            {
                cur = t.to_string();
                continue;
            }
            if node.get("inner").and_then(|i| i.get("module")).is_some() {
                return Some(cur);
            }
            return None;
        }
        None
    }

    /// True when the id names a module (via `paths`, else the index node).
    fn is_module(&self, id: &str) -> bool {
        self.info.get(id).map_or_else(
            || {
                self.node(id)
                    .and_then(|n| n.get("inner"))
                    .and_then(|i| i.get("module"))
                    .is_some()
            },
            |info| info.kind == "module",
        )
    }

    fn is_public(node: &Value) -> bool {
        node.get("visibility").and_then(Value::as_str) == Some("public")
    }

    fn kind_from_key(key: &str) -> Option<ItemKind> {
        Some(match key {
            "function" => ItemKind::Function,
            "struct" => ItemKind::Struct,
            "enum" => ItemKind::Enum,
            "trait" => ItemKind::Trait,
            "struct_field" => ItemKind::Field,
            "variant" => ItemKind::Variant,
            "constant" => ItemKind::Const,
            "static" => ItemKind::Static,
            _ => return None,
        })
    }

    fn kind_from_path(kind: &str) -> Option<ItemKind> {
        Some(match kind {
            "function" => ItemKind::Function,
            "struct" => ItemKind::Struct,
            "enum" => ItemKind::Enum,
            "trait" => ItemKind::Trait,
            "constant" => ItemKind::Const,
            "static" => ItemKind::Static,
            _ => return None,
        })
    }

    /// Stable label for one rustdoc id: canonical path when known, else the
    /// local node's name / kind (generic params, impls, anonymous items).
    fn id_label(&self, id: &str, alias_of: &HashMap<String, String>) -> String {
        if let Some(p) = self.info.get(id) {
            return p.dotted.clone();
        }
        if let Some(node) = self.node(id) {
            if let Some(name) = node.get("name").and_then(Value::as_str) {
                return format!("local:{name}");
            }
            if let Some(inner) = node.get("inner").and_then(Value::as_object) {
                if let Some(imp) = inner.get("impl") {
                    return format!("impl:{}", self.impl_descriptor(imp, alias_of));
                }
                if let Some(kind) = inner.keys().next() {
                    return format!("local:{kind}");
                }
            }
        }
        "unresolved".to_owned()
    }

    /// Canonical, id-free descriptor of an impl block (trait + self type).
    fn impl_descriptor(&self, imp: &Value, alias_of: &HashMap<String, String>) -> String {
        let trait_path = imp
            .get("trait")
            .and_then(|t| t.get("path"))
            .and_then(Value::as_str)
            .unwrap_or("inherent");
        let for_ty = imp
            .get("for")
            .map(|f| compact(&self.normalize(f, alias_of)))
            .unwrap_or_default();
        format!("{trait_path} for {for_ty}")
    }

    /// Recursively replace volatile rustdoc ids with stable labels.
    fn normalize(&self, v: &Value, alias_of: &HashMap<String, String>) -> Value {
        match v {
            Value::Object(m) => {
                let mut out = Map::new();
                let id = m.get("id").and_then(Value::as_u64);
                let canonical = id
                    .and_then(|id| self.info.get(&id.to_string()))
                    .map(|p| p.dotted.as_str());
                for (k, val) in m {
                    if k == "id" && m.contains_key("path") {
                        // A resolved path keeps its canonical spelling; the
                        // numeric id is rebuild churn.
                        continue;
                    }
                    if k == "path" && canonical.is_some() {
                        // Prefer the defining crate's canonical path over the
                        // author-relative spelling (`crate::io::X` vs
                        // `zorro_io::X`) so a rename is reported once, at the
                        // path level, not as signature noise.
                        let c = canonical.unwrap_or_default();
                        let mapped = alias_of.get(c).map_or(c, String::as_str);
                        out.insert(k.clone(), Value::String(mapped.to_owned()));
                    } else if k == "id" && val.is_u64() {
                        out.insert(
                            k.clone(),
                            Value::String(self.id_label(&val.as_u64().unwrap_or(0).to_string(), alias_of)),
                        );
                    } else if matches!(
                        k.as_str(),
                        "impls" | "implementations" | "items" | "fields" | "variants" | "links"
                    ) && val.is_array()
                    {
                        let mut labels: Vec<String> = val
                            .as_array()
                            .map(|a| {
                                a.iter()
                                    .filter_map(Value::as_u64)
                                    .map(|id| self.id_label(&id.to_string(), alias_of))
                                    .collect()
                            })
                            .unwrap_or_default();
                        // Impl/associated-item lists are order-insensitive;
                        // field/variant lists keep their declaration order.
                        if matches!(k.as_str(), "impls" | "implementations" | "items") {
                            labels.sort();
                            labels.dedup();
                        }
                        out.insert(
                            k.clone(),
                            Value::Array(labels.into_iter().map(Value::String).collect()),
                        );
                    } else {
                        out.insert(k.clone(), self.normalize(val, alias_of));
                    }
                }
                Value::Object(out)
            }
            Value::Array(a) => Value::Array(a.iter().map(|e| self.normalize(e, alias_of)).collect()),
            other => other.clone(),
        }
    }

    fn sig(&self, payload: &Value, alias_of: &HashMap<String, String>) -> String {
        compact(&self.normalize(payload, alias_of))
    }

    /// Emit `target` at the exact alias path (named re-export).
    fn emit_target(&self, target: &str, alias: &str, out: &mut BTreeMap<String, RawItem>, depth: usize) {
        if depth > 16 {
            return;
        }
        if self.is_module(target) {
            self.expand_module(target, alias, out, depth + 1, true);
        } else {
            self.emit_alias(target, alias, out);
        }
    }

    /// Emit the public contents of a module `target` at `prefix` (glob).
    fn expand_target(
        &self,
        target: &str,
        prefix: &str,
        out: &mut BTreeMap<String, RawItem>,
        depth: usize,
    ) {
        if depth > 16 {
            return;
        }
        if self.is_module(target) {
            self.expand_module(target, prefix, out, depth + 1, false);
        } else if let Some(info) = self.info.get(target) {
            // Enum glob (`use Enum::*`): variants carry paths.
            self.expand_path_prefix(&info.dotted, prefix, out, false);
        }
    }

    fn expand_module(
        &self,
        target: &str,
        prefix: &str,
        out: &mut BTreeMap<String, RawItem>,
        depth: usize,
        collapse_dup: bool,
    ) {
        if let Some(info) = self.info.get(target) {
            if info.crate_id == 0 {
                self.expand_local_module(target, prefix, out, depth);
            } else {
                self.expand_path_prefix(&info.dotted, prefix, out, collapse_dup);
            }
        } else {
            self.expand_local_module(target, prefix, out, depth);
        }
    }

    /// Expand every `paths` entry under `base` to the same suffix under
    /// `prefix` (external modules; also enums whose variants have paths).
    fn expand_path_prefix(
        &self,
        base: &str,
        prefix: &str,
        out: &mut BTreeMap<String, RawItem>,
        collapse_dup: bool,
    ) {
        let base_colon = format!("{base}::");
        let alias_last = prefix.rsplit("::").next().unwrap_or(prefix);
        for (id, info) in &self.info {
            let Some(mut suffix) = info.dotted.strip_prefix(&base_colon) else {
                continue;
            };
            if info.kind == "module" {
                continue;
            }
            // `pub use zorro_io as io;` over a crate that also has an `io`
            // module flattens to `zorro::io::X`, not `zorro::io::io::X`.
            // Drop a leading segment that just repeats the alias name.
            if collapse_dup {
                if let Some(rest) = suffix.strip_prefix(alias_last) {
                    if let Some(rest) = rest.strip_prefix("::") {
                        suffix = rest;
                    }
                }
            }
            let alias = format!("{prefix}::{suffix}");
            self.emit_alias_with_target(id, &alias, &info.dotted, out);
        }
    }

    /// Walk a local module's `index` children. `prefix` is where its public
    /// contents land (the module itself is not emitted, matching the direct
    /// producer's module policy).
    fn expand_local_module(
        &self,
        module: &str,
        prefix: &str,
        out: &mut BTreeMap<String, RawItem>,
        depth: usize,
    ) {
        if depth > 16 {
            return;
        }
        let Some(children) = self
            .node(module)
            .and_then(|n| n.get("inner"))
            .and_then(|i| i.get("module"))
            .and_then(|m| m.get("items"))
            .and_then(Value::as_array)
        else {
            return;
        };
        for child in children.iter().filter_map(Value::as_u64) {
            let cid = child.to_string();
            let Some(node) = self.node(&cid) else { continue };
            if let Some(parts) = self.use_parts(&cid) {
                // Private `use` bindings are not re-exported by a glob.
                if !Self::is_public(node) {
                    continue;
                }
                if parts.is_glob || parts.name.is_none() {
                    if let Some(t) = parts.target.and_then(|t| self.resolve_target(t)) {
                        self.expand_target(&t, prefix, out, depth + 1);
                    }
                } else if let Some(t) = parts.target.and_then(|t| self.resolve_target(t)) {
                    let alias = format!("{prefix}::{}", parts.name.unwrap_or_default());
                    self.emit_target(&t, &alias, out, depth + 1);
                }
                continue;
            }
            let Some(inner) = node.get("inner").and_then(Value::as_object) else {
                continue;
            };
            let Some(kind_key) = inner.keys().next() else { continue };
            let Some(name) = node.get("name").and_then(Value::as_str) else {
                continue;
            };
            if kind_key == "module" {
                if Self::is_public(node) {
                    self.expand_local_module(&cid, &format!("{prefix}::{name}"), out, depth + 1);
                }
                continue;
            }
            if !Self::is_public(node) {
                continue;
            }
            let alias = format!("{prefix}::{name}");
            self.emit_alias(&cid, &alias, out);
        }
    }

    /// Insert one alias item. Signature rendering is deferred until every
    /// alias is known (see `parse_doc`), so signature paths can prefer the
    /// local re-export spelling over the defining crate's.
    fn emit_alias(&self, target_id: &str, alias: &str, out: &mut BTreeMap<String, RawItem>) {
        if let Some(info) = self.info.get(target_id) {
            self.emit_alias_with_target(target_id, alias, &info.dotted, out);
            return;
        }
        if out.contains_key(alias) {
            return;
        }
        let Some(node) = self.node(target_id) else { return };
        let Some(inner) = node.get("inner").and_then(Value::as_object) else { return };
        let Some((kind_key, payload)) = inner.iter().next() else { return };
        let Some(kind) = Self::kind_from_key(kind_key) else { return };
        let mut attrs = collect_attrs(node);
        if !attrs.iter().any(|a| a == RE_EXPORT_ATTR) {
            attrs.push(RE_EXPORT_ATTR.to_owned());
        }
        out.insert(
            alias.to_owned(),
            RawItem {
                path: alias.to_owned(),
                kind,
                payload: Some(payload.clone()),
                target_path: None,
                attrs,
            },
        );
    }

    fn emit_alias_with_target(
        &self,
        target_id: &str,
        alias: &str,
        target_path: &str,
        out: &mut BTreeMap<String, RawItem>,
    ) {
        if out.contains_key(alias) {
            return;
        }
        let Some(info) = self.info.get(target_id) else { return };
        let Some(kind) = Self::kind_from_path(&info.kind) else {
            return;
        };
        let node = self.node(target_id);
        let payload = node
            .and_then(|n| n.get("inner"))
            .and_then(Value::as_object)
            .and_then(|m| m.values().next())
            .cloned();
        let mut attrs = node.map(collect_attrs).unwrap_or_default();
        if !attrs.iter().any(|a| a == RE_EXPORT_ATTR) {
            attrs.push(RE_EXPORT_ATTR.to_owned());
        }
        out.insert(
            alias.to_owned(),
            RawItem {
                path: alias.to_owned(),
                kind,
                payload,
                target_path: Some(target_path.to_owned()),
                attrs,
            },
        );
    }

    fn module_path_of(&self, module: &str) -> String {
        self.module_path
            .get(module)
            .cloned()
            .unwrap_or_else(|| self.root.clone())
    }
}

/// An item before signature rendering: payload is `Some` for in-crate items
/// (direct or local alias), `None` for external aliases (path-only).
struct RawItem {
    path: String,
    kind: ItemKind,
    payload: Option<Value>,
    target_path: Option<String>,
    attrs: Vec<String>,
}

struct UseParts<'a> {
    #[allow(dead_code)]
    source: &'a str,
    name: Option<&'a str>,
    target: Option<u64>,
    is_glob: bool,
}

fn parse_doc(root: &Value, version: &str) -> Result<ApiSnapshot, ProducerError> {
    let index = root
        .get("index")
        .and_then(Value::as_object)
        .ok_or_else(|| ProducerError::BadShape("missing `index` object".to_owned()))?;
    let paths = root
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| ProducerError::BadShape("missing `paths` object".to_owned()))?;

    let root_id = root.get("root").and_then(Value::as_u64).map(|n| n.to_string());
    let doc = Doc::new(index, paths, root_id.as_deref());
    // Path → item; last direct item wins, aliases never overwrite direct items.
    let mut by_path: BTreeMap<String, RawItem> = BTreeMap::new();

    for (id, node) in index {
        if !Doc::is_public(node) {
            continue;
        }
        let Some(info) = doc.info.get(id) else { continue };
        if info.crate_id != 0 {
            continue;
        }
        let Some(inner) = node.get("inner").and_then(Value::as_object) else {
            continue;
        };
        let mut inner_iter = inner.iter();
        let Some((kind_key, payload)) = inner_iter.next() else {
            continue;
        };
        if inner_iter.next().is_some() {
            continue;
        }
        let Some(kind) = Doc::kind_from_key(kind_key) else {
            continue;
        };
        by_path.insert(
            info.dotted.clone(),
            RawItem {
                path: info.dotted.clone(),
                kind,
                payload: Some(payload.clone()),
                target_path: None,
                attrs: collect_attrs(node),
            },
        );
    }

    // Materialize public `use` items in publicly reachable modules. Uses in
    // private modules are covered by a glob expansion of that module.
    for (id, node) in index {
        if !Doc::is_public(node) {
            continue;
        }
        let Some(parts) = doc.use_parts(id) else { continue };
        let Some(parent) = doc.parent.get(id) else { continue };
        if !doc.module_path.contains_key(parent) {
            continue;
        }
        let prefix = doc.module_path_of(parent);
        let Some(target) = parts.target.and_then(|t| doc.resolve_target(t)) else {
            continue;
        };
        if parts.is_glob || parts.name.is_none() {
            doc.expand_target(&target, &prefix, &mut by_path, 0);
        } else {
            let alias = format!("{prefix}::{}", parts.name.unwrap_or_default());
            doc.emit_target(&target, &alias, &mut by_path, 0);
        }
    }

    // Prefer the shortest local alias for each canonical target: signature
    // paths then use the public spelling (`zorro::backend::ComputeBackend`)
    // rather than the defining crate's (`zorro_nn::backend::ComputeBackend`).
    let mut alias_of: HashMap<String, String> = HashMap::new();
    for raw in by_path.values() {
        let Some(target) = &raw.target_path else { continue };
        match alias_of.get(target) {
            Some(current) if current.len() <= raw.path.len() => {}
            _ => {
                alias_of.insert(target.clone(), raw.path.clone());
            }
        }
    }

    let items: Vec<Item> = by_path
        .into_values()
        .map(|raw| {
            let sig = match &raw.payload {
                Some(payload) => doc.sig(payload, &alias_of),
                None => format!(
                    "re-export of {}",
                    raw.target_path.as_deref().unwrap_or_default()
                ),
            };
            Item {
                path: raw.path,
                kind: raw.kind,
                sig,
                attrs: raw.attrs,
            }
        })
        .collect();
    Ok(ApiSnapshot::new(version, items))
}

/// Attribute markers for one index node: `deprecated` when `deprecation` is
/// non-null, `non_exhaustive` when the attrs list mentions it (covers both
/// the string and object encodings rustdoc has used). Unknown attrs ignored —
/// `Item.attrs` records presence of the markers `classify` reads, not full
/// syntax.
fn collect_attrs(node: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if node.get("deprecation").is_some_and(|d| !d.is_null()) {
        out.push("deprecated".to_owned());
    }
    if let Some(attrs) = node.get("attrs").and_then(Value::as_array) {
        for a in attrs {
            let text = match a {
                Value::String(s) => s.clone(),
                Value::Object(m) => m.keys().next().cloned().unwrap_or_default(),
                _ => String::new(),
            };
            if text.contains("non_exhaustive") && !out.contains(&"non_exhaustive".to_owned()) {
                out.push("non_exhaustive".to_owned());
            }
            if text.contains("deprecated") && !out.contains(&"deprecated".to_owned()) {
                out.push("deprecated".to_owned());
            }
        }
    }
    out
}

/// Compact JSON rendering of a rustdoc `inner` payload (the sig).
fn compact(v: &Value) -> String {
    let mut buf = Vec::new();
    write_compact(v, &mut buf);
    String::from_utf8(buf).unwrap_or_default()
}

fn write_compact(v: &Value, out: &mut Vec<u8>) {
    // Structural chars by hand so key order follows the input document;
    // string escaping delegated to serde_json.
    match v {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
        Value::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Value::String(s) => {
            let esc = serde_json::Value::String(s.clone()).to_string();
            out.extend_from_slice(esc.as_bytes());
        }
        Value::Array(a) => {
            out.push(b'[');
            for (i, e) in a.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_compact(e, out);
            }
            out.push(b']');
        }
        Value::Object(m) => {
            out.push(b'{');
            for (i, (k, e)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                let esc = serde_json::Value::String(k.clone()).to_string();
                out.extend_from_slice(esc.as_bytes());
                out.push(b':');
                write_compact(e, out);
            }
            out.push(b'}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal rustdoc-JSON-shaped doc: index + paths, mixed visibility.
    fn mini_doc() -> String {
        serde_json::json!({
            "index": {
                "10": {"visibility": "public", "inner": {"function": {"sig": "fn a()"}}},
                "20": {"visibility": "public", "inner": {"struct": {"fields": []}}},
                "30": {"visibility": "default", "inner": {"function": {"sig": "fn priv()"}}},
                "40": {"visibility": "public", "inner": {"module": {"items": []}}},
                "50": {"visibility": "public", "inner": {"function": {"sig": "fn ext()"}}},
                "60": {"visibility": "public", "inner": {"weird_future_kind": {}}},
                "70": {"visibility": "public"}
            },
            "paths": {
                "10": {"crate_id": 0, "path": ["demo", "a"]},
                "20": {"crate_id": 0, "path": ["demo", "T"]},
                "30": {"crate_id": 0, "path": ["demo", "priv"]},
                "40": {"crate_id": 0, "path": ["demo", "sub"]},
                "50": {"crate_id": 99, "path": ["std", "ext"]},
                "60": {"crate_id": 0, "path": ["demo", "future"]},
                "70": {"crate_id": 0, "path": ["demo", "noshape"]}
            }
        })
        .to_string()
    }

    #[test]
    fn keeps_public_in_crate_known_kinds() {
        let s = snapshot_from_rustdoc_str(&mini_doc(), "demo 0.1.0").unwrap();
        let paths: Vec<&str> = s.items.iter().map(|i| i.path.as_str()).collect();
        assert_eq!(paths, vec!["demo::T", "demo::a"]);
        assert_eq!(s.version, "demo 0.1.0");
    }

    #[test]
    fn sig_change_in_payload_flips_diff() {
        let a = snapshot_from_rustdoc_str(&mini_doc(), "v1").unwrap();
        let evolved = mini_doc().replace("fn a()", "fn a(x: u8)");
        let b = snapshot_from_rustdoc_str(&evolved, "v2").unwrap();
        let d = crate::diff::diff_snapshots(&a, &b);
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].path, "demo::a");
        assert!(d.added.is_empty() && d.removed.is_empty());
    }

    #[test]
    fn bad_shape_errors_not_panics() {
        let err = snapshot_from_rustdoc_str(r#"{"no": "tables"}"#, "v").unwrap_err();
        assert!(matches!(err, ProducerError::BadShape(_)));
        assert!(snapshot_from_rustdoc_str("not json", "v").is_err());
    }

    #[test]
    fn missing_file_errors() {
        let err = snapshot_from_rustdoc_json(std::path::Path::new("/nonexistent/x.json"), "v")
            .unwrap_err();
        assert!(matches!(err, ProducerError::Io(_)));
    }

    #[test]
    fn attrs_flow_from_nodes() {
        let doc = serde_json::json!({
            "index": {
                "1": {"visibility": "public",
                      "deprecation": {"note": "use b instead"},
                      "inner": {"function": {"sig": "fn a()"}}},
                "2": {"visibility": "public",
                      "attrs": ["non_exhaustive"],
                      "inner": {"struct": {}}},
            },
            "paths": {
                "1": {"crate_id": 0, "path": ["demo", "a"]},
                "2": {"crate_id": 0, "path": ["demo", "T"]},
            }
        })
        .to_string();
        let s = snapshot_from_rustdoc_str(&doc, "v").unwrap();
        assert!(s.get("demo::a").unwrap().has_attr("deprecated"));
        assert!(s.get("demo::T").unwrap().has_attr("non_exhaustive"));
        // And they move severity: deprecated removal still breaks but says so.
        let d = crate::diff::diff_snapshots(&s, &crate::snapshot::ApiSnapshot::new("v2", vec![]));
        let breaks = crate::classify::classify_diff(&d);
        assert!(breaks.iter().any(|b| b.note.contains("deprecated")));
    }

    /// `pub use ext_mod::Thing;` surfaces at the local path with the stable
    /// external marker, not at the defining crate's path.
    #[test]
    fn named_reexport_is_materialized() {
        let doc = serde_json::json!({
            "index": {
                "1": {"visibility": "public", "inner": {"module": {"items": [2]}}},
                "2": {"visibility": "public",
                      "inner": {"use": {"source": "ext_mod::Thing", "name": "Thing",
                                        "id": 10, "is_glob": false}}}
            },
            "paths": {
                "1": {"crate_id": 0, "path": ["demo"], "kind": "module"},
                "10": {"crate_id": 7, "path": ["ext_mod", "Thing"], "kind": "struct"}
            }
        })
        .to_string();
        let s = snapshot_from_rustdoc_str(&doc, "v").unwrap();
        let it = s.get("demo::Thing").expect("re-export surfaced");
        assert_eq!(it.kind, ItemKind::Struct);
        assert!(it.has_attr(RE_EXPORT_ATTR));
        assert_eq!(it.sig, "re-export of ext_mod::Thing");
    }

    /// `pub use ext as ext_alias;` expands the whole external module.
    #[test]
    fn module_alias_expands_contents() {
        let doc = serde_json::json!({
            "index": {
                "1": {"visibility": "public", "inner": {"module": {"items": [2]}}},
                "2": {"visibility": "public",
                      "inner": {"use": {"source": "ext", "name": "ext", "id": 10,
                                        "is_glob": false}}}
            },
            "paths": {
                "1": {"crate_id": 0, "path": ["demo"], "kind": "module"},
                "10": {"crate_id": 7, "path": ["ext"], "kind": "module"},
                "11": {"crate_id": 7, "path": ["ext", "A"], "kind": "struct"},
                "12": {"crate_id": 7, "path": ["ext", "sub", "B"], "kind": "enum"}
            }
        })
        .to_string();
        let s = snapshot_from_rustdoc_str(&doc, "v").unwrap();
        assert!(s.get("demo::ext::A").is_some());
        assert!(s.get("demo::ext::sub::B").is_some());
    }

    /// Globs, chains and private local modules: `pub use private_mod::*;`
    /// walks the index tree (private modules have no `paths` entry).
    #[test]
    fn glob_of_private_local_module_materializes() {
        let doc = serde_json::json!({
            "index": {
                "1": {"visibility": "public", "inner": {"module": {"items": [2]}}},
                "2": {"visibility": "public",
                      "inner": {"use": {"source": "inner", "name": null, "id": 5,
                                        "is_glob": true}}},
                "5": {"visibility": "default", "name": "inner",
                      "inner": {"module": {"items": [6, 7]}}},
                "6": {"visibility": "public", "name": "Api",
                      "inner": {"struct": {"kind": {"unit": null}, "generics": {}}}},
                "7": {"visibility": "default", "name": "hidden",
                      "inner": {"function": {"sig": "fn hidden()"}}}
            },
            "paths": {
                "1": {"crate_id": 0, "path": ["demo"], "kind": "module"}
            }
        })
        .to_string();
        let s = snapshot_from_rustdoc_str(&doc, "v").unwrap();
        assert!(s.get("demo::Api").is_some());
        assert!(s.get("demo::hidden").is_none());
    }

    /// The same shape with different rustdoc ids must not report a diff.
    #[test]
    fn id_churn_does_not_change_signatures() {
        let doc = |ids: [u64; 2]| {
            serde_json::json!({
                "index": {
                    "1": {"visibility": "public",
                          "inner": {"function": {"sig": {"inputs": [
                              ["x", {"resolved_path": {"path": "ext::Thing",
                                                       "id": ids[0], "args": null}}]],
                              "output": null}}}},
                    "2": {"visibility": "public",
                          "inner": {"struct": {"kind": {"plain": {"fields": [], "has_stripped_fields": false}},
                                               "impls": [ids[1], ids[0]]}}}
                },
                "paths": {
                    "1": {"crate_id": 0, "path": ["demo", "f"], "kind": "function"},
                    "2": {"crate_id": 0, "path": ["demo", "S"], "kind": "struct"},
                    "10": {"crate_id": 9, "path": ["ext", "Thing"], "kind": "struct"},
                    "11": {"crate_id": 9, "path": ["ext", "Other"], "kind": "struct"},
                    "99": {"crate_id": 9, "path": ["ext", "Thing"], "kind": "struct"},
                    "77": {"crate_id": 9, "path": ["ext", "Other"], "kind": "struct"}
                }
            })
            .to_string()
        };
        let a = snapshot_from_rustdoc_str(&doc([10, 11]), "v1").unwrap();
        let b = snapshot_from_rustdoc_str(&doc([99, 77]), "v2").unwrap();
        let d = crate::diff::diff_snapshots(&a, &b);
        assert!(d.is_empty(), "id churn produced a diff: {d:?}");
    }
}
