//! Language detection by marker files. Scans the repo root and up to two
//! directory levels below it, skipping dependency and build directories.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lang {
    Rust,
    TypeScript,
    Vue,
    Svelte,
    Python,
}

impl Lang {
    pub fn key(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::TypeScript => "ts",
            Lang::Vue => "vue",
            Lang::Svelte => "svelte",
            Lang::Python => "python",
        }
    }
}

#[derive(Debug, Default)]
pub struct Detection {
    pub langs: BTreeSet<Lang>,
    /// Marker file → language, relative to the repo root, for the report.
    pub evidence: Vec<(PathBuf, Lang)>,
    /// Markers seen but not supported (e.g. plain-JS `package.json`).
    pub ignored: Vec<(PathBuf, &'static str)>,
}

const SKIP: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "dist",
    "build",
    "vendor",
    ".worktrees",
    ".cache",
    "artifacts",
    ".beads",
    ".svelte-kit",
    ".nuxt",
    "__pycache__",
];
const MAX_DEPTH: usize = 2;

pub fn detect(root: &Path) -> Detection {
    let mut d = Detection::default();
    walk(root, root, 0, &mut d);
    d
}

fn walk(root: &Path, dir: &Path, depth: usize, d: &mut Detection) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut subdirs = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        if path.is_dir() {
            if depth < MAX_DEPTH && !SKIP.contains(&name.as_str()) && !name.starts_with('.') {
                subdirs.push(path);
            }
            continue;
        }
        match name.as_str() {
            "Cargo.toml" => d.add(rel, Lang::Rust),
            "pyproject.toml" | "requirements.txt" | "setup.py" | "setup.cfg" | "uv.lock" => d.add(rel, Lang::Python),
            "package.json" => node(dir, rel, d),
            _ => {}
        }
    }
    subdirs.sort();
    for s in subdirs {
        walk(root, &s, depth + 1, d);
    }
}

fn node(dir: &Path, rel: PathBuf, d: &mut Detection) {
    let deps = package_deps(&dir.join("package.json"));
    let has = |n: &str| deps.iter().any(|x| x == n);
    let mut hit = false;
    if has("vue") || has("nuxt") {
        d.add(rel.clone(), Lang::Vue);
        hit = true;
    }
    if has("svelte") || has("@sveltejs/kit") {
        d.add(rel.clone(), Lang::Svelte);
        hit = true;
    }
    if hit || has("typescript") || dir.join("tsconfig.json").is_file() {
        d.add(rel, Lang::TypeScript);
    } else {
        d.ignored.push((rel, "package.json without TypeScript"));
    }
}

fn package_deps(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return vec![];
    };
    ["dependencies", "devDependencies", "peerDependencies"]
        .iter()
        .filter_map(|k| json.get(k).and_then(|v| v.as_object()))
        .flat_map(|m| m.keys().cloned())
        .collect()
}

impl Detection {
    fn add(&mut self, rel: PathBuf, lang: Lang) {
        self.langs.insert(lang);
        self.evidence.push((rel, lang));
    }

    pub fn keys(&self) -> Vec<String> {
        self.langs.iter().map(|l| l.key().to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn repo(files: &[(&str, &str)]) -> TempDir {
        let t = TempDir::new().unwrap();
        for (p, c) in files {
            let p = t.path().join(p);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, c).unwrap();
        }
        t
    }

    fn langs(files: &[(&str, &str)]) -> Vec<String> {
        detect(repo(files).path()).keys()
    }

    #[test]
    fn detects_single_languages() {
        assert_eq!(langs(&[("Cargo.toml", "")]), ["rust"]);
        assert_eq!(langs(&[("pyproject.toml", "")]), ["python"]);
        assert_eq!(langs(&[("package.json", "{}"), ("tsconfig.json", "{}")]), ["ts"]);
    }

    #[test]
    fn vue_and_svelte_imply_typescript() {
        assert_eq!(langs(&[("package.json", r#"{"dependencies":{"vue":"^3"}}"#)]), ["ts", "vue"]);
        assert_eq!(langs(&[("package.json", r#"{"devDependencies":{"@sveltejs/kit":"^2"}}"#)]), ["ts", "svelte"]);
    }

    #[test]
    fn plain_js_is_ignored_not_detected() {
        let d = detect(repo(&[("package.json", r#"{"dependencies":{"left-pad":"1"}}"#)]).path());
        assert!(d.langs.is_empty());
        assert_eq!(d.ignored.len(), 1);
    }

    #[test]
    fn detects_mixed_repo_in_subdirs() {
        let got = langs(&[
            ("backend/Cargo.toml", ""),
            ("web/package.json", r#"{"dependencies":{"vue":"3"}}"#),
            ("tools/py/pyproject.toml", ""),
        ]);
        assert_eq!(got, ["rust", "ts", "vue", "python"]);
    }

    #[test]
    fn skips_dependency_dirs_and_depth_limit() {
        assert!(langs(&[("node_modules/x/package.json", r#"{"dependencies":{"vue":"3"}}"#)]).is_empty());
        assert!(langs(&[("target/debug/Cargo.toml", "")]).is_empty());
        assert!(langs(&[("a/b/c/Cargo.toml", "")]).is_empty());
        assert!(langs(&[(".hidden/Cargo.toml", "")]).is_empty());
    }

    #[test]
    fn malformed_package_json_falls_back_to_tsconfig() {
        assert_eq!(langs(&[("package.json", "{not json"), ("tsconfig.json", "")]), ["ts"]);
        assert!(langs(&[("package.json", "{not json")]).is_empty());
    }
}
