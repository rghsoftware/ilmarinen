//! `ilmarinen setup`: per-machine configuration, with no input and no
//! knowledge of the machine or its owner (decision 0013). Runs doctor's tool
//! check first (never prompting) and refuses on failure, then configures both
//! hosts and prints what it did.

use crate::doctor::{self, BM_PROJECT};
use crate::{assets, manifest, repos, util};
use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

#[derive(clap::Args, Default)]
pub struct Args {
    /// Skip the MCP liveness check in the closing doctor run.
    #[arg(long)]
    pub skip_mcp: bool,
    /// Ilmarinen package checkout holding `.claude-plugin/marketplace.json`.
    /// Defaults to the checkout this binary was built from.
    #[arg(long)]
    pub package: Option<PathBuf>,
}

fn package_root(a: &Args) -> Result<PathBuf> {
    if let Some(p) = &a.package {
        return Ok(p.clone());
    }
    let built_from = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    if built_from.join("plugin").is_dir() {
        return Ok(built_from.canonicalize()?);
    }
    bail!("cannot find the Ilmarinen package checkout; pass --package <dir>")
}

pub const OPT_IN_KEYWORD: &str = "full-depth";
pub const INSTRUCTIONS_MAX: usize = 15;
/// Where the instruction files point for pinned memory (the default config dir).
const PINNED_REF: &str = "~/.config/ilmarinen/memory/pinned.md";

/// The personal memory project: inside the package's own config directory.
pub fn memory_dir() -> Result<PathBuf> {
    Ok(util::config_dir()?.join("memory"))
}

pub fn run(a: &Args) -> Result<()> {
    println!("== tools (doctor; no prompts) ==");
    if !doctor::check_tools(&doctor::ToolCheck { offer: false, ..Default::default() })? {
        bail!(
            "doctor failed; setup refuses to continue. Install the missing tools (or run `ilmarinen doctor`, which offers mise) and re-run."
        );
    }
    let home = util::home()?;

    println!("\n== config ==");
    let mut cfg = repos::load_config()?;
    if cfg.opt_in_keyword.as_deref().is_none_or(str::is_empty) {
        cfg.opt_in_keyword = Some(OPT_IN_KEYWORD.into());
        repos::save_config(&cfg)?;
        println!("  wrote    {} (opt_in_keyword = \"{OPT_IN_KEYWORD}\")", repos::config_path()?.display());
    } else {
        println!("  ok       {}", repos::config_path()?.display());
    }
    let keyword = cfg.opt_in_keyword.clone().unwrap_or_else(|| OPT_IN_KEYWORD.into());
    create_if_absent(&repos::repos_path()?, repos::REPOS_HEADER)?;
    create_if_absent(
        &util::config_dir()?.join("leak-patterns.txt"),
        "# Optional extra fixed strings for the leak guard, one per line. Starts empty;\n# Ilmarinen never writes here.\n",
    )?;

    println!("\n== personal memory (how you work; never what you own) ==");
    let mem = memory_dir()?;
    std::fs::create_dir_all(&mem)?;
    if !mem.join(".git").is_dir() {
        util::capture("git", &["init", "-q"], Some(&mem))?;
        println!("  git init {} (private; no remote)", mem.display());
    }
    create_if_absent(&mem.join("pinned.md"), "")?;
    basic_memory_project(&home, &mem)?;

    println!("\n== instruction files ==");
    let body = instructions(&keyword);
    upsert(&home.join(".claude/CLAUDE.md"), &format!("{body}\n@{PINNED_REF}"))?;
    // OpenCode cannot inject SessionStart context (converter gap `session-start-context`).
    let oc_body = format!(
        "{body}\n- At session start in a repo with `.beads/`, read `~/.cache/ilmarinen/handoff/<repo name>.md` if it exists (else `bd recall ilmarinen-handoff`) and run `bd ready` before anything else."
    );
    upsert(&home.join(".config/opencode/AGENTS.md"), &oc_body)?;
    edit_opencode_json(&home, |cfg| {
        let list = cfg.entry("instructions").or_insert_with(|| json!([]));
        let pin = json!(PINNED_REF);
        match list.as_array_mut() {
            Some(a) if !a.contains(&pin) => {
                a.push(pin);
                Ok(true)
            }
            Some(_) => Ok(false),
            None => bail!("opencode.json `instructions` is not an array"),
        }
    })?;

    println!("\n== plugin and MCP servers (user scope, both hosts) ==");
    let package = package_root(a)?;
    claude_plugin(&package)?;
    opencode_layout(&home, &package)?;

    println!("\n== telemetry opt-outs ==");
    telemetry_opt_out()?;

    println!("\n== doctor (full) ==");
    let r = doctor::run(&doctor::Args { skip_mcp: a.skip_mcp, yes: false })?;
    if !r.passed() {
        println!("setup finished, but doctor still reports problems (above).");
    }
    Ok(())
}

fn create_if_absent(p: &Path, content: &str) -> Result<()> {
    if p.exists() {
        println!("  exists   {}", p.display());
    } else {
        util::write(p, content)?;
        println!("  created  {}", p.display());
    }
    Ok(())
}

fn upsert(p: &Path, body: &str) -> Result<()> {
    let cur = std::fs::read_to_string(p).unwrap_or_default();
    let new = util::upsert_block(&cur, body);
    if new == cur {
        println!("  ok       {}", p.display());
    } else {
        util::write(p, new)?;
        println!("  updated  {} (ilmarinen block only)", p.display());
    }
    Ok(())
}

/// The ≤ 15-line user-level instruction block shared by both hosts.
pub fn instructions(keyword: &str) -> String {
    let s = format!(
        "## Ilmarinen (applies to every repo)\n\
- Pinned memory (`{PINNED_REF}`, Basic Memory project `{BM_PROJECT}`) loads at session start; read it if it has not.\n\
- Basic Memory holds how I work: preferences that are not workflow rules, recurring decisions and their reasons, tooling lessons that are not repo-specific, standing instructions about me as a collaborator. Write to it only when I explicitly ask you to remember something.\n\
- Never store what I own (identity, usernames, hostnames, hardware, network, paths, accounts, deployment targets, anything credential-adjacent) in memory or in a repo. If a session needs such a fact, run a command and do not persist the result.\n\
- Route every request as `trivial`, `standard` or `major` before starting. The word `{keyword}` opts a request into `major`.\n\
- Tasks live in Beads (`bd`), never in markdown TODOs."
    );
    debug_assert!(s.lines().count() <= INSTRUCTIONS_MAX);
    s
}

/// Pinned Basic Memory invocation (see docs/tool-notes.md for why it is pinned).
pub const BASIC_MEMORY_UVX: &[&str] = &[
    "--from",
    "basic-memory==0.23.2",
    "--with",
    "fastmcp==4.0.0b1",
    "--with",
    "fastmcp-slim==4.0.0b1",
    "basic-memory",
];

fn basic_memory_project(home: &Path, mem: &Path) -> Result<()> {
    if doctor::bm_project_exists(home, mem) {
        println!("  ok       Basic Memory project {BM_PROJECT:?}");
        return Ok(());
    }
    let mem_s = mem.to_string_lossy();
    let mut args: Vec<&str> = BASIC_MEMORY_UVX.to_vec();
    args.extend(["project", "add", BM_PROJECT, &mem_s]);
    util::run("uvx", &args, None).context("registering the Basic Memory project")?;
    println!("  added    Basic Memory project {BM_PROJECT:?} → {}", mem.display());
    Ok(())
}

/// Load, mutate and (if changed) save ~/.config/opencode/opencode.json with a
/// one-time backup. Refuses to touch a JSONC-only config.
fn edit_opencode_json(home: &Path, f: impl FnOnce(&mut Map<String, Value>) -> Result<bool>) -> Result<()> {
    let dir = home.join(".config/opencode");
    let p = dir.join("opencode.json");
    if !p.exists() && dir.join("opencode.jsonc").exists() {
        bail!("only opencode.jsonc exists; add the entries by hand (setup does not rewrite JSONC comments)");
    }
    let mut v: Value = match std::fs::read_to_string(&p) {
        Ok(t) => serde_json::from_str(&t).with_context(|| format!("parsing {}", p.display()))?,
        Err(_) => json!({"$schema": "https://opencode.ai/config.json"}),
    };
    let obj = v.as_object_mut().context("opencode.json is not an object")?;
    if f(obj)? {
        let bak = dir.join("opencode.json.ilmarinen-backup");
        if p.exists() && !bak.exists() {
            std::fs::copy(&p, &bak)?;
            println!("  backup   {}", bak.display());
        }
        util::write(&p, serde_json::to_string_pretty(&v)? + "\n")?;
        println!("  updated  {}", p.display());
    } else {
        println!("  ok       {}", p.display());
    }
    Ok(())
}

/// Claude `.mcp.json` server entry → OpenCode `mcp` entry.
pub fn to_opencode(server: &Value) -> Result<Value> {
    let env_conv = |s: &str| -> String {
        // ${VAR} / ${VAR:-default} → {env:VAR}; OpenCode has no default syntax.
        let re = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)(:-[^}]*)?\}").unwrap();
        re.replace_all(s, "{env:$1}").into_owned()
    };
    let conv_map = |m: Option<&Value>| -> Value {
        let mut out = Map::new();
        if let Some(Value::Object(m)) = m {
            for (k, v) in m {
                out.insert(k.clone(), json!(env_conv(v.as_str().unwrap_or_default())));
            }
        }
        Value::Object(out)
    };
    if let Some(url) = server.get("url").and_then(Value::as_str) {
        let mut o = json!({"type": "remote", "url": env_conv(url), "enabled": true});
        if server.get("headers").is_some() {
            o["headers"] = conv_map(server.get("headers"));
        }
        return Ok(o);
    }
    let cmd = server.get("command").and_then(Value::as_str).context("MCP server has neither url nor command")?;
    let mut command = vec![json!(env_conv(cmd))];
    if let Some(Value::Array(args)) = server.get("args") {
        command.extend(args.iter().map(|a| json!(env_conv(a.as_str().unwrap_or_default()))));
    }
    let mut o = json!({"type": "local", "command": command, "enabled": true});
    if server.get("env").is_some() {
        o["environment"] = conv_map(server.get("env"));
    }
    Ok(o)
}

/// Claude Code: install the plugin at user scope; its own `.mcp.json` starts
/// the servers (decision 0011, Observed Consequences). Never `claude mcp add`.
fn claude_plugin(package: &Path) -> Result<()> {
    if !package.join(".claude-plugin/marketplace.json").is_file() {
        println!("  {} has no .claude-plugin/marketplace.json yet (Phase 2); skipping", package.display());
        return Ok(());
    }
    let listed = util::sh("claude plugin list", None).map(|o| o.text).unwrap_or_default();
    if listed.contains("ilmarinen@ilmarinen") {
        println!("  ok       Claude Code plugin ilmarinen@ilmarinen");
        return Ok(());
    }
    util::run("claude", &["plugin", "marketplace", "add", &package.to_string_lossy()], None)?;
    util::run("claude", &["plugin", "install", "ilmarinen@ilmarinen", "-s", "user"], None)?;
    println!("  added    Claude Code plugin ilmarinen@ilmarinen (user scope)");
    Ok(())
}

/// OpenCode: link the converted layout (`hosts/opencode/`) into
/// `~/.config/opencode/` and merge its `mcp` block into the global opencode.json.
fn opencode_layout(home: &Path, package: &Path) -> Result<()> {
    let src = package.join("hosts/opencode");
    if !src.join("opencode.json").is_file() {
        println!("  {} not generated; run `just convert` in the package", src.display());
        return Ok(());
    }
    let dst = home.join(".config/opencode");
    let mut links = vec![
        (src.join("plugins/ilmarinen.ts"), dst.join("plugins/ilmarinen.ts")),
        (src.join("ilmarinen"), dst.join("ilmarinen")),
    ];
    for e in std::fs::read_dir(src.join("skills"))?.flatten() {
        links.push((e.path(), dst.join("skills").join(e.file_name())));
    }
    for (from, to) in &links {
        link(from, to)?;
    }
    let v: Value = serde_json::from_str(&std::fs::read_to_string(src.join("opencode.json"))?)?;
    let servers = v.get("mcp").and_then(Value::as_object).context("hosts/opencode/opencode.json has no mcp")?.clone();
    edit_opencode_json(home, |cfg| {
        let mcp = cfg.entry("mcp").or_insert_with(|| json!({}));
        let mcp = mcp.as_object_mut().context("opencode.json `mcp` is not an object")?;
        let mut changed = false;
        for (name, want) in &servers {
            if mcp.get(name) != Some(want) {
                mcp.insert(name.clone(), want.clone());
                changed = true;
            }
        }
        Ok(changed)
    })
}

/// Symlink `to` → `from`. Leaves anything that is not already our link alone.
pub fn link(from: &Path, to: &Path) -> Result<()> {
    match std::fs::read_link(to) {
        Ok(cur) if cur == from => {
            println!("  ok       {}", to.display());
            return Ok(());
        }
        Ok(_) | Err(_) if to.symlink_metadata().is_ok() => {
            println!("  ! kept   {} (exists and is not Ilmarinen's link; resolve by hand)", to.display());
            return Ok(());
        }
        _ => {}
    }
    if let Some(p) = to.parent() {
        std::fs::create_dir_all(p)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(from, to).with_context(|| format!("linking {}", to.display()))?;
    println!("  linked   {} → {}", to.display(), from.display());
    Ok(())
}

/// Merge `env` into the `env` object of a JSON settings file, keeping everything else.
pub fn set_json_env(path: &Path, env: &std::collections::BTreeMap<String, String>) -> Result<()> {
    let mut v: Value = match std::fs::read_to_string(path) {
        Ok(t) => serde_json::from_str(&t).with_context(|| format!("parsing {}", path.display()))?,
        Err(_) => json!({}),
    };
    let obj = v.as_object_mut().with_context(|| format!("{} is not an object", path.display()))?;
    let e = obj.entry("env").or_insert_with(|| json!({}));
    let e = e.as_object_mut().with_context(|| format!("{} `env` is not an object", path.display()))?;
    let mut changed = false;
    for (k, val) in env {
        if e.get(k).and_then(Value::as_str) != Some(val) {
            e.insert(k.clone(), json!(val));
            changed = true;
        }
    }
    if changed {
        util::write(path, serde_json::to_string_pretty(&v)? + "\n")?;
    }
    Ok(())
}

fn telemetry_opt_out() -> Result<()> {
    for t in manifest::parse(&assets::tools_toml())? {
        let Some(tel) = &t.telemetry else { continue };
        if let Some(file) = &tel.json_env {
            set_json_env(&util::expand_home(file)?, &tel.env)?;
            println!("  ✔ {:<12} env opt-outs in {file}", t.name);
            continue;
        }
        let unset: Vec<&str> = tel.env.keys().map(String::as_str).collect();
        match &tel.disable {
            // The per-run env must not mask the command's own effect.
            Some(cmd) => match util::sh_without(cmd, &unset) {
                Some(o) if o.ok => println!("  ✔ {:<12} `{cmd}`", t.name),
                _ => println!("  ! {:<12} `{cmd}` failed (is {} installed?)", t.name, t.name),
            },
            None => println!("  ! {:<12} no persistent opt-out; per-run env only", t.name),
        }
    }
    let unconfirmed = doctor::check_telemetry(false)?;
    if !unconfirmed.is_empty() {
        println!("  unconfirmed: {}", unconfirmed.join(", "));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructions_fit_budget_and_name_keyword() {
        let s = instructions(OPT_IN_KEYWORD);
        assert!(s.lines().count() <= INSTRUCTIONS_MAX);
        assert!(s.contains("`full-depth`"));
        assert!(s.contains("Never store what I own"));
    }

    #[test]
    fn converts_mcp_entries_for_opencode() {
        let stdio = json!({"type":"stdio","command":"uvx","args":["x","--k","${HOME}/p"],"env":{"A":"${TOKEN:-}"}});
        assert_eq!(
            to_opencode(&stdio).unwrap(),
            json!({"type":"local","command":["uvx","x","--k","{env:HOME}/p"],"enabled":true,"environment":{"A":"{env:TOKEN}"}})
        );
        let http = json!({"type":"http","url":"https://m.example/mcp","headers":{"Authorization":"Bearer ${K}"}});
        assert_eq!(
            to_opencode(&http).unwrap(),
            json!({"type":"remote","url":"https://m.example/mcp","enabled":true,"headers":{"Authorization":"Bearer {env:K}"}})
        );
        assert!(to_opencode(&json!({})).is_err());
    }

    #[test]
    fn json_env_merge_keeps_other_keys() {
        let t = tempfile::TempDir::new().unwrap();
        let p = t.path().join("settings.json");
        std::fs::write(&p, r#"{"model":"x","env":{"KEEP":"1"}}"#).unwrap();
        let want: std::collections::BTreeMap<String, String> =
            [("DISABLE_TELEMETRY".to_string(), "1".to_string())].into();
        assert!(!doctor::json_env_has(&p, &want));
        set_json_env(&p, &want).unwrap();
        assert!(doctor::json_env_has(&p, &want));
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(v["model"], "x");
        assert_eq!(v["env"]["KEEP"], "1");
    }

    #[test]
    fn link_is_idempotent_and_never_clobbers() {
        let t = tempfile::TempDir::new().unwrap();
        let from = t.path().join("src");
        std::fs::create_dir(&from).unwrap();
        let to = t.path().join("dst/skills/route");
        link(&from, &to).unwrap();
        link(&from, &to).unwrap();
        assert_eq!(std::fs::read_link(&to).unwrap(), from);
        let mine = t.path().join("dst/skills/plan");
        std::fs::create_dir_all(&mine).unwrap();
        link(&from, &mine).unwrap();
        assert!(std::fs::read_link(&mine).is_err(), "a user's own directory is left alone");
    }
}
