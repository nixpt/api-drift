//! `api-drift.toml` consumer configuration (APIDRIFT-9).
//!
//! ```toml
//! [[upstream]]
//! name    = "bro-tui"
//! surface = "backend-events"
//! source  = { path = "../bro-tui/api-drift" }
//! pin     = "sha256:a17f936…"
//! ```

use serde::Deserialize;
use std::path::Path;

/// Parsed `api-drift.toml`.
#[derive(Debug, Deserialize)]
pub struct ConsumerConfig {
    #[serde(default)]
    pub upstream: Vec<UpstreamPin>,
}

/// One upstream surface pin.
#[derive(Debug, Deserialize)]
pub struct UpstreamPin {
    /// Human name, e.g. `bro-tui`.
    pub name: String,
    /// Ledger key, e.g. `backend-events`.
    pub surface: String,
    /// Where to read the upstream ledger directory.
    pub source: UpstreamSource,
    /// `sha256:…` hash of the last-known-good snapshot (empty = from beginning).
    #[serde(default)]
    pub pin: String,
}

/// How to locate the upstream ledger directory.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UpstreamSource {
    /// Relative or absolute filesystem path, e.g. `{ path = "../bro-tui/api-drift" }`.
    Path { path: String },
}

/// Parse `api-drift.toml` at `path`.
pub fn read_config(path: &Path) -> Result<ConsumerConfig, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}
