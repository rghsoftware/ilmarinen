//! Package files compiled into the binary, so `init` needs nothing from the
//! network. Rule: init must succeed without network; network is used only for
//! opt-in sync features and never for telemetry.

use crate::render::Source;
use include_dir::{Dir, DirEntry, include_dir};
use std::path::Path;

const TOOLS_TOML: &str = include_str!("../../tools.toml");
const MISE_TOML: &str = include_str!("../../mise.toml");

/// Test hook: a path in this env var replaces the embedded `tools.toml`.
pub const TOOLS_ENV: &str = "ILMARINEN_TOOLS_TOML";
/// Test hook: a path in this env var replaces the embedded `mise.toml`.
pub const MISE_ENV: &str = "ILMARINEN_MISE_TOML";

fn embedded_or(env: &str, embedded: &'static str) -> String {
    std::env::var_os(env).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_else(|| embedded.to_string())
}

/// The tool manifest (embedded unless `ILMARINEN_TOOLS_TOML` names a file).
pub fn tools_toml() -> String {
    embedded_or(TOOLS_ENV, TOOLS_TOML)
}

/// The pinned mise config (embedded unless `ILMARINEN_MISE_TOML` names a file).
pub fn mise_toml() -> String {
    embedded_or(MISE_ENV, MISE_TOML)
}
pub static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../templates");
pub static PLUGIN: Dir = include_dir!("$CARGO_MANIFEST_DIR/../plugin");

/// `plugin/.mcp.json`, if the plugin has been built (Phase 2).
pub fn mcp_json() -> Option<&'static str> {
    PLUGIN.get_file(".mcp.json").and_then(|f| f.contents_utf8())
}

/// The leak guard, stamped into each repo as `scripts/leak-guard.sh` so the
/// git pre-commit hook and the plugin run the same script.
pub fn leak_guard() -> &'static str {
    PLUGIN
        .get_file("hooks/leak-guard.sh")
        .and_then(|f| f.contents_utf8())
        .expect("plugin/hooks/leak-guard.sh is embedded")
}

/// Scripts under `templates/` that must be stamped executable.
fn is_executable(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "sh")
}

pub fn template_sources() -> Vec<Source<'static>> {
    let mut out = Vec::new();
    collect(&TEMPLATES, &mut out);
    out
}

fn collect(dir: &'static Dir, out: &mut Vec<Source<'static>>) {
    for e in dir.entries() {
        match e {
            DirEntry::Dir(d) => collect(d, out),
            DirEntry::File(f) => {
                out.push(Source { path: f.path(), bytes: f.contents(), executable: is_executable(f.path()) })
            }
        }
    }
}

/// Stable hash of the embedded templates; recorded in `.ilmarinen.version`.
pub fn templates_digest() -> String {
    let mut srcs = template_sources();
    srcs.sort_by(|a, b| a.path.cmp(b.path));
    let mut buf = Vec::new();
    for s in srcs {
        buf.extend_from_slice(s.path.to_string_lossy().as_bytes());
        buf.push(0);
        buf.extend_from_slice(s.bytes);
        buf.push(0);
    }
    crate::util::sha256(&buf)[..16].to_string()
}
