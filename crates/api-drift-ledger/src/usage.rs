//! `syn`-based usage scanner: find call sites of ledger item paths in the
//! consumer's source tree (APIDRIFT-9).
//!
//! Matches by the last two path segments (type + member), so both `use`d and
//! fully-qualified paths are found:
//!   `BackendEvent::Output(s)` and `bro_tui::backend::BackendEvent::Output(s)`
//! both match item path `bro_tui::backend::BackendEvent::Output`.

use std::path::Path;
use syn::visit::Visit;

/// A single call-site where a ledger item is referenced.
#[derive(Debug, Clone)]
pub struct UsageSite {
    /// Full ledger item path, e.g. `bro_tui::backend::BackendEvent::Output`.
    pub item_path: String,
    /// Source file relative to the scan root, e.g. `src/agent.rs`.
    pub file: String,
    /// 1-indexed line number (from `proc-macro2` span-locations).
    pub line: u32,
}

/// Scan all `.rs` files under `src_dir` for usages of `item_paths`.
///
/// `src_dir`'s parent is used as the root for relative file names.
pub fn scan(src_dir: &Path, item_paths: &[&str]) -> Vec<UsageSite> {
    let targets = build_targets(item_paths);
    let mut sites = Vec::new();
    let root = src_dir.parent().unwrap_or(src_dir);
    walk(src_dir, root, &targets, &mut sites);
    sites
}

/// `(type_segment, member_segment, full_item_path)` triples built from paths.
fn build_targets(item_paths: &[&str]) -> Vec<(String, String, String)> {
    item_paths
        .iter()
        .filter_map(|p| {
            let segs: Vec<&str> = p.split("::").collect();
            if segs.len() < 2 {
                return None;
            }
            Some((
                segs[segs.len() - 2].to_owned(),
                segs[segs.len() - 1].to_owned(),
                (*p).to_owned(),
            ))
        })
        .collect()
}

fn walk(dir: &Path, root: &Path, targets: &[(String, String, String)], sites: &mut Vec<UsageSite>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, root, targets, sites);
        } else if path.extension().is_some_and(|e| e == "rs") {
            scan_file(&path, root, targets, sites);
        }
    }
}

fn scan_file(
    path: &Path,
    root: &Path,
    targets: &[(String, String, String)],
    sites: &mut Vec<UsageSite>,
) {
    let Ok(src) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(file) = syn::parse_file(&src) else {
        return;
    };
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();
    let mut visitor = UsageVisitor {
        targets,
        file: rel,
        found: Vec::new(),
    };
    visitor.visit_file(&file);
    sites.extend(visitor.found);
}

struct UsageVisitor<'a> {
    targets: &'a [(String, String, String)],
    file: String,
    found: Vec<UsageSite>,
}

impl<'ast> Visit<'ast> for UsageVisitor<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segs: Vec<_> = path.segments.iter().collect();
        if segs.len() >= 2 {
            let type_seg = segs[segs.len() - 2].ident.to_string();
            let member_seg = segs[segs.len() - 1].ident.to_string();
            for (enum_s, variant_s, full_path) in self.targets {
                if type_seg == *enum_s && member_seg == *variant_s {
                    let line = u32::try_from(segs[segs.len() - 1].ident.span().start().line)
                        .unwrap_or(u32::MAX);
                    self.found.push(UsageSite {
                        item_path: full_path.clone(),
                        file: self.file.clone(),
                        line,
                    });
                }
            }
        }
        syn::visit::visit_path(self, path);
    }
}
