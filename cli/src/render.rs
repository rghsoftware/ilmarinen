//! Renders `templates/` for one repo.
//!
//! Conventions:
//! - A path segment `@<keys>/` (e.g. `@rust/`, `@ts,vue/`) includes the subtree
//!   only when one of the comma-separated keys is true; the segment is dropped
//!   from the output path.
//! - Files ending `.tmpl` are rendered and lose the suffix; others are copied.
//! - Inside `.tmpl`: `<%var%>` substitutes; `<%#if a|b%>` … `<%else%>` …
//!   `<%/if%>` on their own lines select blocks (nesting allowed). A key is true
//!   when it is a detected language or a non-empty variable. (`<% %>` because
//!   justfiles and GitHub Actions use `{{ }}` themselves.)
//! - `.gitignore` / `.gitattributes` are merged line by line, never replaced.

use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Whole file owned by the template.
    Whole,
    /// Created when absent, then owned by the repo (SpecScore indexes, tunable
    /// config); `init` never rewrites it and `upgrade` never diffs it.
    Seed,
    /// Append missing lines under a marker; never remove user lines.
    MergeLines,
}

#[derive(Debug, Clone)]
pub struct Rendered {
    pub path: PathBuf,
    pub content: String,
    pub strategy: Strategy,
    pub executable: bool,
}

pub struct Ctx {
    pub vars: BTreeMap<String, String>,
    pub langs: BTreeSet<String>,
}

impl Ctx {
    fn truthy(&self, key: &str) -> bool {
        self.langs.contains(key) || self.vars.get(key).is_some_and(|v| !v.is_empty())
    }

    fn any(&self, keys: &str) -> bool {
        keys.split(['|', ',']).map(str::trim).any(|k| self.truthy(k))
    }
}

pub struct Source<'a> {
    pub path: &'a Path,
    pub bytes: &'a [u8],
    pub executable: bool,
}

pub fn render_all(sources: &[Source], ctx: &Ctx) -> Result<Vec<Rendered>> {
    let mut out = Vec::new();
    for s in sources {
        let Some(path) = output_path(s.path, ctx) else {
            continue;
        };
        if path.file_name().is_some_and(|n| n == ".gitkeep") {
            continue;
        }
        let (path, content) = if path.extension().is_some_and(|e| e == "tmpl") {
            let text = std::str::from_utf8(s.bytes)?;
            (path.with_extension(""), render(text, ctx).map_err(|e| e.context(s.path.display().to_string()))?)
        } else {
            (path, String::from_utf8_lossy(s.bytes).into_owned())
        };
        let strategy = match path.file_name().and_then(|n| n.to_str()) {
            Some(".gitignore" | ".gitattributes") => Strategy::MergeLines,
            _ if is_seed(&path) => Strategy::Seed,
            _ => Strategy::Whole,
        };
        out.push(Rendered { path, content, strategy, executable: s.executable });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Files a repo owns after the first stamp: everything under `spec/`
/// (SpecScore rewrites its indexes), `specscore.yaml` and `scorecard.toml`.
fn is_seed(p: &Path) -> bool {
    p.starts_with("spec") || p == Path::new("specscore.yaml") || p == Path::new("scorecard.toml")
}

fn output_path(p: &Path, ctx: &Ctx) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for c in p.components() {
        let s = c.as_os_str().to_string_lossy();
        if let Some(keys) = s.strip_prefix('@') {
            if !ctx.any(keys) {
                return None;
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

pub fn render(text: &str, ctx: &Ctx) -> Result<String> {
    // Stack of (this branch active, any earlier branch taken, parent active).
    let mut stack: Vec<(bool, bool)> = Vec::new();
    let active = |st: &Vec<(bool, bool)>| st.iter().all(|(a, _)| *a);
    let mut out = String::new();
    for (n, line) in text.split_inclusive('\n').enumerate() {
        let t = line.trim();
        if let Some(cond) = t.strip_prefix("<%#if ").and_then(|r| r.strip_suffix("%>")) {
            let v = ctx.any(cond);
            stack.push((v, v));
            continue;
        }
        if t == "<%else%>" {
            let Some(top) = stack.last_mut() else { bail!("line {}: <%else%> without <%#if%>", n + 1) };
            *top = (!top.1, true);
            continue;
        }
        if t == "<%/if%>" {
            if stack.pop().is_none() {
                bail!("line {}: <%/if%> without <%#if%>", n + 1);
            }
            continue;
        }
        if active(&stack) {
            out.push_str(&substitute(line, ctx).map_err(|e| e.context(format!("line {}", n + 1)))?);
        }
    }
    if !stack.is_empty() {
        bail!("unclosed <%#if%>");
    }
    Ok(out)
}

fn substitute(line: &str, ctx: &Ctx) -> Result<String> {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find("<%") {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find("%>") else { bail!("unclosed <%") };
        let key = rest[i + 2..i + j].trim();
        match ctx.vars.get(key) {
            Some(v) => out.push_str(v),
            None => bail!("unknown template variable {key:?}"),
        }
        rest = &rest[i + j + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

pub const MERGE_MARKER: &str = "# --- ilmarinen ---";

/// Lines of `ours` missing from `existing`, appended under the marker.
pub fn merge_lines(existing: &str, ours: &str) -> String {
    let have: BTreeSet<&str> = existing.lines().map(str::trim).collect();
    let missing: Vec<&str> = ours
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#') && !have.contains(l.trim()))
        .collect();
    if missing.is_empty() {
        return existing.to_string();
    }
    let mut out = existing.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !have.contains(MERGE_MARKER) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(MERGE_MARKER);
        out.push('\n');
    }
    for l in missing {
        out.push_str(l);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(langs: &[&str]) -> Ctx {
        let mut vars = BTreeMap::new();
        vars.insert("name".into(), "demo".into());
        vars.insert("empty".into(), String::new());
        Ctx { vars, langs: langs.iter().map(|s| s.to_string()).collect() }
    }

    #[test]
    fn substitutes_and_selects_blocks() {
        let t = "# <%name%>\n<%#if rust%>\ncargo\n<%else%>\nno-cargo\n<%/if%>\n<%#if ts|python%>\nnode-or-py\n<%/if%>\n<%#if empty%>\nnever\n<%/if%>\njust {{x}} ${{ y }}\nend\n";
        assert_eq!(render(t, &ctx(&["rust"])).unwrap(), "# demo\ncargo\njust {{x}} ${{ y }}\nend\n");
        assert_eq!(render(t, &ctx(&["python"])).unwrap(), "# demo\nno-cargo\nnode-or-py\njust {{x}} ${{ y }}\nend\n");
    }

    #[test]
    fn nested_blocks() {
        let t = "<%#if ts%>\na\n<%#if vue%>\nb\n<%/if%>\n<%/if%>\n";
        assert_eq!(render(t, &ctx(&["ts"])).unwrap(), "a\n");
        assert_eq!(render(t, &ctx(&["ts", "vue"])).unwrap(), "a\nb\n");
        assert_eq!(render(t, &ctx(&["vue"])).unwrap(), "");
    }

    #[test]
    fn errors_on_unknown_var_and_unbalanced() {
        assert!(render("<%nope%>\n", &ctx(&[])).is_err());
        assert!(render("<%#if rust%>\n", &ctx(&[])).is_err());
        assert!(render("<%/if%>\n", &ctx(&[])).is_err());
    }

    #[test]
    fn language_paths_and_suffixes() {
        let srcs = [
            Source { path: Path::new("AGENTS.md.tmpl"), bytes: b"<%name%>\n", executable: false },
            Source { path: Path::new("@rust/clippy.toml"), bytes: b"x", executable: false },
            Source { path: Path::new("@ts,python/scripts/a.sh"), bytes: b"y", executable: true },
            Source { path: Path::new(".gitignore"), bytes: b"artifacts/\n", executable: false },
            Source { path: Path::new("spec/.gitkeep"), bytes: b"", executable: false },
            Source { path: Path::new("spec/features/README.md"), bytes: b"# idx\n", executable: false },
        ];
        let out = render_all(&srcs, &ctx(&["python"])).unwrap();
        let paths: Vec<_> = out.iter().map(|r| r.path.to_string_lossy().into_owned()).collect();
        assert_eq!(paths, [".gitignore", "AGENTS.md", "scripts/a.sh", "spec/features/README.md"]);
        assert_eq!(out[3].strategy, Strategy::Seed);
        assert_eq!(out[0].strategy, Strategy::MergeLines);
        assert_eq!(out[1].content, "demo\n");
        assert!(out[2].executable);
    }

    #[test]
    fn merge_lines_appends_only_missing() {
        let merged = merge_lines("target/\nartifacts/\n", "# ours\nartifacts/\n.worktrees/\n");
        assert_eq!(merged, "target/\nartifacts/\n\n# --- ilmarinen ---\n.worktrees/\n");
        assert_eq!(merge_lines(&merged, ".worktrees/\n"), merged);
        let again = merge_lines(&merged, ".cache/\n");
        assert_eq!(again.matches(MERGE_MARKER).count(), 1);
        assert!(again.ends_with(".worktrees/\n.cache/\n"));
    }
}
