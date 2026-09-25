//! `ilmarinen init <repo>` stamps `templates/`; `ilmarinen upgrade <repo>`
//! diffs the current templates against what was stamped (via
//! `.ilmarinen.version`) and applies only with confirmation.

use crate::detect::{self, Lang};
use crate::render::{self, Ctx, Rendered, Strategy};
use crate::{assets, doctor, repos, util};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MARKER: &str = ".ilmarinen.version";

#[derive(clap::Args)]
pub struct InitArgs {
    pub repo: PathBuf,
    /// Git host for specscore.yaml when there is no `origin` remote.
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub org: Option<String>,
    #[arg(long = "name")]
    pub repo_name: Option<String>,
    /// Skip `bd init` (e.g. fixtures in CI that only check templates).
    #[arg(long)]
    pub no_beads: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Marker {
    pub version: String,
    pub templates: String,
    pub languages: Vec<String>,
    /// Path → sha256 of the content Ilmarinen last wrote there.
    #[serde(default)]
    pub files: BTreeMap<String, String>,
}

fn toplevel(repo: &Path) -> Result<PathBuf> {
    let out = util::capture("git", &["-C", &repo.to_string_lossy(), "rev-parse", "--show-toplevel"], None)
        .with_context(|| format!("{} is not inside a git repository", repo.display()))?;
    Ok(PathBuf::from(out.trim()))
}

struct Identity {
    host: String,
    org: String,
    repo: String,
}

fn identity(root: &Path, a: Option<&InitArgs>) -> Result<Identity> {
    let flags = a.map(|a| (a.host.clone(), a.org.clone(), a.repo_name.clone()));
    if let Some((Some(host), Some(org), Some(repo))) = flags {
        return Ok(Identity { host, org, repo });
    }
    if let Ok(url) = util::capture("git", &["remote", "get-url", "origin"], Some(root))
        && let Some((host, org, repo)) = repos::remote_id(&url)
    {
        return Ok(Identity { host, org, repo });
    }
    let dir = root.file_name().map_or("repo".into(), |n| n.to_string_lossy().into_owned());
    println!(
        "No usable `origin` remote; using local/local/{dir} in specscore.yaml (pass --host/--org/--name to set it)."
    );
    Ok(Identity { host: "local".into(), org: "local".into(), repo: dir })
}

fn stack_line(langs: &[Lang]) -> String {
    let has = |l| langs.contains(&l);
    let mut parts = Vec::new();
    if has(Lang::Rust) {
        parts.push("Rust".to_string());
    }
    if has(Lang::TypeScript) {
        let mut fw = Vec::new();
        if has(Lang::Vue) {
            fw.push("Vue");
        }
        if has(Lang::Svelte) {
            fw.push("Svelte");
        }
        parts.push(if fw.is_empty() { "TypeScript".into() } else { format!("TypeScript ({})", fw.join(", ")) });
    }
    if has(Lang::Python) {
        parts.push("Python".to_string());
    }
    if parts.is_empty() { "none detected".into() } else { parts.join(", ") }
}

fn plan(root: &Path, id: &Identity, keyword: &str) -> Result<(Vec<Rendered>, Vec<String>)> {
    let det = detect::detect(root);
    let langs: Vec<Lang> = det.langs.iter().copied().collect();
    println!("Detected: {}", stack_line(&langs));
    for (p, l) in &det.evidence {
        println!("  {} → {}", p.display(), l.key());
    }
    for (p, why) in &det.ignored {
        println!("  {} ignored: {why}", p.display());
    }
    let mut vars = BTreeMap::new();
    vars.insert("project".into(), id.repo.clone());
    vars.insert("host".into(), id.host.clone());
    vars.insert("org".into(), id.org.clone());
    vars.insert("repo".into(), id.repo.clone());
    vars.insert("stack".into(), stack_line(&langs));
    vars.insert("keyword".into(), keyword.to_string());
    vars.insert("version".into(), env!("CARGO_PKG_VERSION").into());
    let mut keys = det.keys();
    keys.retain(|k| k != "vue" && k != "svelte");
    keys.push("spec".into());
    vars.insert("langs".into(), keys.join(" | "));
    for (var, lang) in [("rust_dir", Lang::Rust), ("ts_dir", Lang::TypeScript), ("py_dir", Lang::Python)] {
        vars.insert(var.into(), lang_dir(&det, lang));
    }
    let typecheck = if langs.contains(&Lang::Vue) {
        "vue-tsc --noEmit"
    } else if langs.contains(&Lang::Svelte) {
        "svelte-check"
    } else {
        "tsc --noEmit"
    };
    vars.insert("ts_typecheck".into(), typecheck.into());
    for (k, v) in doctor::mise_pins()? {
        vars.insert(format!("pin:{k}"), v);
    }
    let ctx = Ctx { vars, langs: det.keys().into_iter().collect() };
    let mut files = render::render_all(&assets::template_sources(), &ctx)?;
    files.push(Rendered {
        path: "scripts/leak-guard.sh".into(),
        content: assets::leak_guard().to_string(),
        strategy: Strategy::Whole,
        executable: true,
    });
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((files, det.keys()))
}

/// Directory (relative to the repo root) of the shallowest marker for `lang`.
fn lang_dir(det: &detect::Detection, lang: Lang) -> String {
    det.evidence
        .iter()
        .filter(|(_, l)| *l == lang)
        .map(|(p, _)| p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default())
        .min_by_key(|d| (d.matches('/').count(), d.clone()))
        .map(|d| if d.is_empty() { ".".into() } else { d })
        .unwrap_or_else(|| ".".into())
}

pub const GUARD_BEGIN: &str = "# --- BEGIN ILMARINEN LEAK GUARD (managed by `ilmarinen init`) ---";
const GUARD: &str = r#"# --- BEGIN ILMARINEN LEAK GUARD (managed by `ilmarinen init`) ---
# Runs before anything else in this hook; a failure aborts the whole commit.
_ilm_root=$(git rev-parse --show-toplevel)
if [ -x "$_ilm_root/scripts/leak-guard.sh" ]; then
  "$_ilm_root/scripts/leak-guard.sh" </dev/null || exit 1
fi
# --- END ILMARINEN LEAK GUARD ---
"#;

/// Put the leak guard first in the repo's pre-commit hook (wherever
/// `core.hooksPath` points). Idempotent; keeps every other line.
pub fn wire_leak_guard(root: &Path) -> Result<PathBuf> {
    let dir = util::capture("git", &["rev-parse", "--path-format=absolute", "--git-path", "hooks"], Some(root))?;
    let file = PathBuf::from(dir.trim()).join("pre-commit");
    let cur = std::fs::read_to_string(&file).unwrap_or_default();
    let new = guard_first(&cur);
    if new != cur {
        util::write(&file, &new)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(file)
}

/// Hook text with our section directly after the shebang, before any other
/// section (e.g. Beads' integration block).
pub fn guard_first(cur: &str) -> String {
    let without = match (cur.find(GUARD_BEGIN), cur.find("# --- END ILMARINEN LEAK GUARD ---\n")) {
        (Some(b), Some(e)) => format!("{}{}", &cur[..b], &cur[e + "# --- END ILMARINEN LEAK GUARD ---\n".len()..]),
        _ => cur.to_string(),
    };
    let (shebang, rest) = match without.strip_prefix("#!") {
        Some(_) => without
            .split_once('\n')
            .map(|(a, b)| (format!("{a}\n"), b.to_string()))
            .unwrap_or((without.clone(), String::new())),
        None => ("#!/usr/bin/env sh\n".to_string(), without.clone()),
    };
    format!("{shebang}{GUARD}{rest}")
}

fn keyword() -> Result<String> {
    Ok(repos::load_config()?
        .opt_in_keyword
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| crate::setup::OPT_IN_KEYWORD.to_string()))
}

fn read_marker(root: &Path) -> Result<Option<Marker>> {
    let p = root.join(MARKER);
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(toml::from_str(&std::fs::read_to_string(&p)?).with_context(|| format!("parsing {}", p.display()))?))
}

fn write_file(root: &Path, r: &Rendered, content: &str) -> Result<()> {
    let p = root.join(&r.path);
    util::write(&p, content)?;
    #[cfg(unix)]
    if r.executable {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

pub fn init(a: &InitArgs) -> Result<()> {
    let root = toplevel(&a.repo)?;
    let kw = keyword()?;
    let id = identity(&root, Some(a))?;
    let (files, langs) = plan(&root, &id, &kw)?;
    // Beads first: its init commit must hold only its own files (decision 0003).
    if !a.no_beads {
        beads_init(&root)?;
    }
    let mut marker = read_marker(&root)?.unwrap_or_default();
    stamp(&root, &files, &mut marker)?;
    marker.version = env!("CARGO_PKG_VERSION").into();
    marker.templates = assets::templates_digest();
    marker.languages = langs;
    util::write(&root.join(MARKER), toml::to_string(&marker)?)?;
    println!("  wrote    {MARKER}");
    post_init(a, &root, &id)
}

/// Write rendered files that are absent, merge line-merged files, and leave
/// differing files alone. Records what Ilmarinen wrote in `marker.files`.
fn stamp(root: &Path, files: &[Rendered], marker: &mut Marker) -> Result<()> {
    for r in files {
        let p = root.join(&r.path);
        let rel = r.path.to_string_lossy().into_owned();
        let existing = std::fs::read_to_string(&p).ok();
        match (r.strategy, existing) {
            (Strategy::MergeLines, cur) => {
                let cur = cur.unwrap_or_default();
                let merged = render::merge_lines(&cur, &r.content);
                if merged != cur {
                    write_file(root, r, &merged)?;
                    println!("  merged   {rel}");
                } else {
                    println!("  ok       {rel}");
                }
            }
            (Strategy::Seed, Some(_)) => {
                println!("  ok       {rel} (repo-owned)");
                continue;
            }
            (Strategy::Whole | Strategy::Seed, None) => {
                write_file(root, r, &r.content)?;
                println!("  created  {rel}");
            }
            (Strategy::Whole, Some(cur)) if cur == r.content => println!("  ok       {rel}"),
            (Strategy::Whole, Some(_)) => {
                println!("  kept     {rel} (exists and differs; review with `ilmarinen upgrade`)");
                if marker.files.contains_key(&rel) {
                    continue; // keep the hash of what we last wrote
                }
            }
        }
        marker.files.insert(rel, util::sha256(r.content.as_bytes()));
    }
    Ok(())
}

pub const BD_INIT_MSG: &str = "bd init: initialize beads issue tracking";

/// True when nothing is staged (compared with HEAD, or the empty tree on an
/// unborn branch). `bd init` commits whatever is staged along with `.beads/`.
fn index_clean(root: &Path) -> Result<bool> {
    let base = if util::capture("git", &["rev-parse", "--verify", "-q", "HEAD"], Some(root)).is_ok() {
        "HEAD"
    } else {
        "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
    };
    Ok(util::sh(&format!("git diff --cached --quiet {base}"), Some(root)).is_some_and(|o| o.ok))
}

/// Paths `bd init` stages and commits by name, besides `.beads/`
/// (gastownhall/beads v1.3.0, `cmd/bd/init.go`).
pub const BD_STAGED: &[&str] =
    &["AGENTS.md", "CLAUDE.md", ".claude/settings.json", ".gitignore", ".agents", ".codex", ".cursor"];

fn beads_init(root: &Path) -> Result<()> {
    if root.join(".beads").is_dir() {
        println!("  ok       .beads/ exists; bd init skipped");
        return Ok(());
    }
    // Default (non-stealth) mode: bd init commits `.beads/`, anything already
    // staged, and the paths in BD_STAGED. Nothing of the user's may ride along.
    if !index_clean(root)? {
        bail!(
            "the git index has staged changes; bd init would commit them with .beads/. \
             Commit or unstage them (`git restore --staged .`), then re-run `ilmarinen init`."
        );
    }
    let mut args = vec!["status", "--porcelain", "--"];
    args.extend(BD_STAGED);
    let dirty = util::capture("git", &args, Some(root))?;
    if !dirty.trim().is_empty() {
        bail!(
            "bd init commits these paths by name and they have uncommitted changes:\n{dirty}\
             Commit or stash them, then re-run `ilmarinen init`."
        );
    }
    doctor::ensure_opted_out("bd")?;
    // Wired before bd init: Beads copies an existing pre-commit into
    // .beads/hooks and appends its block after ours.
    wire_leak_guard(root)?;
    // --skip-agents: AGENTS.md is ours.
    util::run("bd", &["init", "--non-interactive", "--skip-agents", "--init-if-missing", "-q"], Some(root))
        .context("bd init")?;
    println!("  ran      bd init (it commits .beads/ as \"{BD_INIT_MSG}\")");
    Ok(())
}

fn post_init(_a: &InitArgs, root: &Path, id: &Identity) -> Result<()> {
    let root = root.to_path_buf();
    let hook = wire_leak_guard(&root)?;
    println!("  wired    leak guard first in {}", hook.display());
    if root.join("specscore.yaml").exists() {
        let ok = util::run("specscore", &["spec", "lint"], Some(&root)).is_ok();
        println!("  {}  specscore spec lint", if ok { "passed" } else { "FAILED" });
        if !ok {
            bail!("specscore spec lint failed in {}", root.display());
        }
    }
    let entry = repos::Repo { id: format!("{}/{}/{}", id.host, id.org, id.repo), path: root.clone() };
    let rp = repos::repos_path()?;
    if repos::add_repo(&rp, &entry)? {
        println!("  added    {} to {}", entry.id, rp.display());
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum Change {
    Add,
    /// File untouched since we wrote it; safe to replace.
    Update,
    /// File edited locally since we wrote it; needs its own confirmation.
    Conflict,
    Merge,
}

pub fn classify(current: Option<&str>, recorded: Option<&str>, new: &Rendered) -> Option<Change> {
    if new.strategy == Strategy::Seed {
        return current.is_none().then_some(Change::Add);
    }
    if new.strategy == Strategy::MergeLines {
        let cur = current.unwrap_or("");
        return (render::merge_lines(cur, &new.content) != cur).then_some(Change::Merge);
    }
    match current {
        None => Some(Change::Add),
        Some(c) if c == new.content => None,
        Some(c) if recorded == Some(util::sha256(c.as_bytes()).as_str()) => Some(Change::Update),
        Some(_) => Some(Change::Conflict),
    }
}

pub fn upgrade(repo: &Path) -> Result<()> {
    let root = toplevel(repo)?;
    let Some(mut marker) = read_marker(&root)? else {
        bail!("{} has no {MARKER}; run `ilmarinen init` first", root.display());
    };
    println!(
        "Stamped with ilmarinen {} (templates {}); this is {} (templates {}).",
        marker.version,
        marker.templates,
        env!("CARGO_PKG_VERSION"),
        assets::templates_digest()
    );
    let id = identity(&root, None)?;
    let (files, langs) = plan(&root, &id, &keyword()?)?;
    if langs != marker.languages {
        println!("Languages changed: {:?} → {:?}", marker.languages, langs);
    }

    let mut changes = Vec::new();
    for r in &files {
        let rel = r.path.to_string_lossy().into_owned();
        let cur = std::fs::read_to_string(root.join(&r.path)).ok();
        if let Some(c) = classify(cur.as_deref(), marker.files.get(&rel).map(String::as_str), r) {
            let target = if c == Change::Merge {
                render::merge_lines(cur.as_deref().unwrap_or(""), &r.content)
            } else {
                r.content.clone()
            };
            println!("\n=== {rel} ({c:?})");
            let diff = similar::TextDiff::from_lines(cur.as_deref().unwrap_or(""), &target);
            print!("{}", diff.unified_diff().context_radius(3).header("current", "template"));
            changes.push((r, c, target));
        }
    }
    let rendered: Vec<String> = files.iter().map(|r| r.path.to_string_lossy().into_owned()).collect();
    for gone in marker.files.keys().filter(|k| !rendered.contains(k)) {
        println!("\nno longer in templates: {gone} (left in place; delete it yourself if unwanted)");
    }
    if changes.is_empty() {
        println!("\nUp to date.");
    } else if util::confirm(&format!("\nApply {} change(s)?", changes.len())) {
        for (r, c, target) in &changes {
            let rel = r.path.to_string_lossy().into_owned();
            if *c == Change::Conflict && !util::confirm(&format!("{rel} was edited locally. Overwrite?")) {
                println!("  skipped  {rel}");
                continue;
            }
            write_file(&root, r, target)?;
            marker.files.insert(rel.clone(), util::sha256(r.content.as_bytes()));
            println!("  applied  {rel}");
        }
    } else {
        println!("Nothing applied.");
        return Ok(());
    }
    marker.files.retain(|k, _| rendered.contains(k));
    marker.version = env!("CARGO_PKG_VERSION").into();
    marker.templates = assets::templates_digest();
    marker.languages = langs;
    util::write(&root.join(MARKER), toml::to_string(&marker)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(content: &str, strategy: Strategy) -> Rendered {
        Rendered { path: "f".into(), content: content.into(), strategy, executable: false }
    }

    #[test]
    fn classify_changes() {
        let new = r("v2\n", Strategy::Whole);
        let h1 = util::sha256(b"v1\n");
        assert_eq!(classify(None, None, &new), Some(Change::Add));
        assert_eq!(classify(Some("v2\n"), Some(&h1), &new), None);
        assert_eq!(classify(Some("v1\n"), Some(&h1), &new), Some(Change::Update));
        assert_eq!(classify(Some("mine\n"), Some(&h1), &new), Some(Change::Conflict));
        assert_eq!(classify(Some("v1\n"), None, &new), Some(Change::Conflict));
        let seed = r("template index\n", Strategy::Seed);
        assert_eq!(classify(Some("specscore rewrote this\n"), None, &seed), None);
        assert_eq!(classify(None, None, &seed), Some(Change::Add));
        let gi = r("a/\n", Strategy::MergeLines);
        assert_eq!(classify(Some("a/\n"), None, &gi), None);
        assert_eq!(classify(Some("b/\n"), None, &gi), Some(Change::Merge));
    }

    #[test]
    fn stamp_is_idempotent_and_upgrade_sees_edits() {
        let t = tempfile::TempDir::new().unwrap();
        let root = t.path();
        std::fs::write(root.join(".gitignore"), "target/\n").unwrap();
        std::fs::write(root.join("README.md"), "user readme\n").unwrap();
        let v1 = vec![
            r("# agents v1\n", Strategy::Whole).at("AGENTS.md"),
            r("user readme differs\n", Strategy::Whole).at("README.md"),
            r("artifacts/\n", Strategy::MergeLines).at(".gitignore"),
        ];
        let mut m = Marker::default();
        stamp(root, &v1, &mut m).unwrap();
        let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
        assert_eq!(read("AGENTS.md"), "# agents v1\n");
        assert_eq!(read("README.md"), "user readme\n", "existing file must not be overwritten");
        assert!(read(".gitignore").starts_with("target/\n") && read(".gitignore").contains("artifacts/"));

        // Second stamp changes nothing.
        let before = (read("AGENTS.md"), read(".gitignore"));
        stamp(root, &v1, &mut m).unwrap();
        assert_eq!(before, (read("AGENTS.md"), read(".gitignore")));

        // Template moves to v2: untouched file → Update; locally edited → Conflict.
        let v2 = r("# agents v2\n", Strategy::Whole).at("AGENTS.md");
        let rec = m.files.get("AGENTS.md").map(String::as_str);
        assert_eq!(classify(Some(&read("AGENTS.md")), rec, &v2), Some(Change::Update));
        std::fs::write(root.join("AGENTS.md"), "# agents v1\nlocal note\n").unwrap();
        assert_eq!(classify(Some(&read("AGENTS.md")), rec, &v2), Some(Change::Conflict));
        // The pre-existing README was never ours → Conflict, never silent.
        let rec = m.files.get("README.md").map(String::as_str);
        assert_eq!(classify(Some(&read("README.md")), rec, &v1[1]), Some(Change::Conflict));
    }

    impl Rendered {
        fn at(mut self, p: &str) -> Self {
            self.path = p.into();
            self
        }
    }

    #[test]
    fn leak_guard_goes_first_and_is_idempotent() {
        let beads = "#!/usr/bin/env sh\n# --- BEGIN BEADS INTEGRATION v1.3.0 ---\nbd hooks run pre-commit\n# --- END BEADS INTEGRATION v1.3.0 ---\n";
        let once = guard_first(beads);
        assert!(once.starts_with("#!/usr/bin/env sh\n# --- BEGIN ILMARINEN LEAK GUARD"));
        assert!(once.find("ILMARINEN").unwrap() < once.find("BEADS").unwrap());
        assert_eq!(guard_first(&once), once);
        // A guard that ended up after another block is moved back to the top.
        let late = format!("{beads}{GUARD}");
        assert_eq!(guard_first(&late), once);
        assert!(guard_first("").starts_with("#!/usr/bin/env sh\n# --- BEGIN ILMARINEN"));
    }

    #[test]
    fn stack_descriptions() {
        assert_eq!(stack_line(&[Lang::Rust, Lang::TypeScript, Lang::Vue]), "Rust, TypeScript (Vue)");
        assert_eq!(stack_line(&[]), "none detected");
    }
}
