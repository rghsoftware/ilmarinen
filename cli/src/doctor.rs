//! `ilmarinen doctor`: tools (tools.toml), telemetry opt-outs, MCP liveness in
//! both hosts (plugin/.mcp.json), and personal state. Offers `mise install` for
//! missing mise-manageable tools; never installs without a yes (decision 0006).

use crate::manifest::{self, Tool, Version};
use crate::{assets, repos, util};
use anyhow::{Context, Result, bail};
use regex::Regex;
use std::collections::BTreeMap;

#[derive(clap::Args, Default)]
pub struct Args {
    /// CI mode: check only `ci`/`all` scope tools, and skip the installed
    /// plugin, MCP liveness and personal state (no hosts, no logins in CI).
    #[arg(long)]
    pub skip_mcp: bool,
    /// Answer yes to the `mise install` offer. Explicit consent, for scripts.
    #[arg(long)]
    pub yes: bool,
}

pub struct Report {
    pub tools_ok: bool,
    pub mcp_ok: bool,
    pub personal_ok: bool,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.tools_ok && self.mcp_ok && self.personal_ok
    }
}

/// How `check_tools` behaves: `offer` asks before `mise install` (never in
/// setup), `yes` answers that offer, `ci` checks only `ci`/`all` scope tools.
#[derive(Default)]
pub struct ToolCheck {
    pub yes: bool,
    pub offer: bool,
    pub ci: bool,
}

pub fn run(args: &Args) -> Result<Report> {
    let tools_ok = check_tools(&ToolCheck { yes: args.yes, offer: true, ci: args.skip_mcp })?;
    check_telemetry(args.skip_mcp)?;
    let (mcp_ok, personal_ok) = if args.skip_mcp {
        println!("\nCI mode (--skip-mcp): dev-scope tools, installed plugin, MCP liveness and personal state skipped");
        (true, true)
    } else {
        let plugin_ok = check_plugins()?;
        (check_mcp()? && plugin_ok, check_personal()?)
    };
    let r = Report { tools_ok, mcp_ok, personal_ok };
    println!("\ndoctor: {}", if r.passed() { "pass" } else { "FAIL" });
    Ok(r)
}

#[derive(Debug)]
enum Status {
    Ok(Version),
    TooOld(Version),
    Missing,
}

fn status(t: &Tool) -> Status {
    let min = Version::parse(&t.min).expect("validated by manifest::parse");
    match util::sh(&t.detect, None) {
        Some(o) => match Version::find(&o.text) {
            Some(v) if o.ok && v >= min => Status::Ok(v),
            Some(v) if o.ok => Status::TooOld(v),
            _ => Status::Missing,
        },
        None => Status::Missing,
    }
}

/// Pinned versions from the embedded mise.toml `[tools]` table.
pub fn mise_pins() -> Result<BTreeMap<String, String>> {
    let v: toml::Value = toml::from_str(&assets::mise_toml()).context("parsing mise.toml")?;
    let tools = v.get("tools").and_then(|t| t.as_table()).context("mise.toml has no [tools]")?;
    Ok(tools.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
}

/// Returns true when every required tool is present and new enough.
pub fn check_tools(opts: &ToolCheck) -> Result<bool> {
    let mut tools = manifest::parse(&assets::tools_toml())?;
    tools.retain(|t| !opts.ci || t.in_ci());
    println!("Tools (tools.toml{}):", if opts.ci { ", ci and all scope" } else { "" });
    let mut failing: Vec<&Tool> = Vec::new();
    for t in &tools {
        let s = status(t);
        print_status(t, &s);
        if !matches!(s, Status::Ok(_)) {
            failing.push(t);
        }
    }
    if failing.is_empty() {
        return Ok(true);
    }

    let pins = mise_pins()?;
    let installable: Vec<(&Tool, &String)> =
        failing.iter().filter_map(|t| t.mise.as_ref().and_then(|k| pins.get(k)).map(|v| (*t, v))).collect();
    if opts.offer && !installable.is_empty() && util::which("mise") {
        let list: Vec<String> =
            installable.iter().map(|(t, v)| format!("{}@{v}", t.mise.as_deref().unwrap_or(&t.name))).collect();
        println!("\nmise can install: {}", list.join(", "));
        println!(
            "This writes the pinned versions to ~/.config/mise/conf.d/ilmarinen.toml and runs `mise install` for them."
        );
        if opts.yes || util::confirm("Run mise install now?") {
            mise_install(&list)?;
            failing.retain(|t| !matches!(status(t), Status::Ok(_)));
            if failing.is_empty() {
                println!("All tools present after mise install.");
                return Ok(true);
            }
        }
    }

    println!("\nInstall these yourself, then re-run `ilmarinen doctor`:");
    for t in &failing {
        println!("  {:<12} {}", t.name, t.install);
    }
    Ok(!failing.iter().any(|t| t.required))
}

fn print_status(t: &Tool, s: &Status) {
    let opt = if t.required { "" } else { " (optional)" };
    match s {
        Status::Ok(v) => println!("  ✔ {:<12} {v}", t.name),
        Status::TooOld(v) => println!("  ✘ {:<12} {v} < {}{opt}: {}", t.name, t.min, t.role),
        Status::Missing => {
            let mark = if t.required { "✘" } else { "!" };
            println!("  {mark} {:<12} missing{opt}: {}", t.name, t.role)
        }
    }
}

fn mise_install(specs: &[String]) -> Result<()> {
    let conf = util::home()?.join(".config/mise/conf.d/ilmarinen.toml");
    let header = "# Written by `ilmarinen doctor` with consent. Pinned versions from the Ilmarinen package.\n";
    util::write(&conf, format!("{header}{}", assets::mise_toml()))?;
    println!("wrote {}", conf.display());
    let mut args = vec!["install"];
    args.extend(specs.iter().map(String::as_str));
    util::run("mise", &args, None)
}

/// Report every tool that phones home and whether its opt-out is confirmed.
/// Unconfirmed opt-outs are reported, not failed: `setup` fixes them.
pub fn check_telemetry(ci: bool) -> Result<Vec<String>> {
    let tools = manifest::parse(&assets::tools_toml())?;
    let mut unconfirmed = Vec::new();
    let with: Vec<&Tool> = tools.iter().filter(|t| t.telemetry.is_some() && (!ci || t.in_ci())).collect();
    if with.is_empty() {
        return Ok(unconfirmed);
    }
    println!("\nTelemetry opt-outs:");
    for t in with {
        let tel = t.telemetry.as_ref().unwrap();
        let env: Vec<String> = tel.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let env = if env.is_empty() { String::new() } else { format!(" [per-run: {}]", env.join(" ")) };
        if let Some(file) = &tel.json_env {
            let path = util::expand_home(file)?;
            if json_env_has(&path, &tel.env) {
                println!("  ✔ {:<12} opted out ({file} env){env}", t.name);
            } else {
                println!(
                    "  ! {:<12} NOT confirmed opted out ({}); fix: `ilmarinen setup` writes {file} env{env}",
                    t.name, tel.sends
                );
                unconfirmed.push(t.name.clone());
            }
            continue;
        }
        let Some(cmd) = &tel.confirm else {
            println!("  ! {:<12} cannot confirm: env-only opt-out, no saved setting ({}){env}", t.name, tel.sends);
            unconfirmed.push(t.name.clone());
            continue;
        };
        // Show the saved state, not our own per-run override.
        let unset: Vec<&str> = tel.env.keys().map(String::as_str).collect();
        match util::sh_without(cmd, &unset) {
            Some(o) if confirmed(&o.text, tel.confirmed_if.as_deref(), tel.confirmed_unless.as_deref()) => {
                println!("  ✔ {:<12} opted out{env}", t.name)
            }
            Some(_) => {
                let fix = tel.disable.as_deref().unwrap_or("see tools.toml");
                println!("  ! {:<12} NOT confirmed opted out ({}); fix: {fix}{env}", t.name, tel.sends);
                unconfirmed.push(t.name.clone());
            }
            None => {
                println!("  ! {:<12} cannot confirm: `{cmd}` did not run{env}", t.name);
                unconfirmed.push(t.name.clone());
            }
        }
    }
    Ok(unconfirmed)
}

/// Make sure the named tool's persistent telemetry opt-out is in effect:
/// confirm it, run its `disable` command if needed, confirm again.
/// Used before invoking a tool so no step relies on `setup` having run.
pub fn ensure_opted_out(name: &str) -> Result<()> {
    let tools = manifest::parse(&assets::tools_toml())?;
    let Some(tel) = tools.iter().find(|t| t.name == name).and_then(|t| t.telemetry.as_ref()) else {
        return Ok(());
    };
    let (Some(confirm), Some(disable)) = (&tel.confirm, &tel.disable) else {
        return Ok(());
    };
    let unset: Vec<&str> = tel.env.keys().map(String::as_str).collect();
    let is_off = || {
        util::sh_without(confirm, &unset)
            .is_some_and(|o| confirmed(&o.text, tel.confirmed_if.as_deref(), tel.confirmed_unless.as_deref()))
    };
    if is_off() {
        return Ok(());
    }
    println!("  {name}: telemetry not confirmed off; running `{disable}`");
    util::sh_without(disable, &unset).filter(|o| o.ok).with_context(|| format!("`{disable}` failed"))?;
    if !is_off() {
        bail!("{name}: `{confirm}` still does not show telemetry off after `{disable}`");
    }
    Ok(())
}

/// Whether the JSON file's `env` object carries every `want` pair.
pub fn json_env_has(path: &std::path::Path, want: &std::collections::BTreeMap<String, String>) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    want.iter().all(|(k, val)| v.get("env").and_then(|e| e.get(k)).and_then(|x| x.as_str()) == Some(val))
}

pub fn confirmed(output: &str, must: Option<&str>, must_not: Option<&str>) -> bool {
    let m = must.is_none_or(|r| Regex::new(r).is_ok_and(|re| re.is_match(output)));
    let n = must_not.is_none_or(|r| Regex::new(r).is_ok_and(|re| !re.is_match(output)));
    m && n
}

/// Server names declared in plugin/.mcp.json.
pub fn mcp_servers() -> Result<Option<Vec<String>>> {
    let Some(text) = assets::mcp_json() else {
        return Ok(None);
    };
    let v: serde_json::Value = serde_json::from_str(text).context("parsing plugin/.mcp.json")?;
    let servers = v.get("mcpServers").and_then(|m| m.as_object()).context("plugin/.mcp.json has no mcpServers")?;
    Ok(Some(servers.keys().cloned().collect()))
}

fn check_mcp() -> Result<bool> {
    println!("\nMCP liveness (plugin/.mcp.json):");
    let Some(servers) = mcp_servers()? else {
        println!("  plugin/.mcp.json not built into this binary yet; nothing to check");
        return Ok(true);
    };
    let mut ok = true;
    // Claude Code lists plugin servers as `plugin:<plugin>:<server>`; other
    // plugins may declare a server with the same bare name.
    for (host, cmd, prefix) in
        [("Claude Code", "claude mcp list", "plugin:ilmarinen:"), ("OpenCode", "opencode mcp list", "")]
    {
        let Some(out) = util::sh(cmd, None) else {
            println!("  ✘ {host}: `{cmd}` did not run");
            ok = false;
            continue;
        };
        for s in &servers {
            let st = mcp_line_status(&out.text, &format!("{prefix}{s}"));
            println!("  {} {host:<11} {s}: {st:?}", if st == Mcp::Connected { "✔" } else { "✘" });
            ok &= st == Mcp::Connected;
        }
    }
    Ok(ok)
}

#[derive(Debug, serde::Deserialize)]
pub struct PluginInfo {
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: serde_json::Map<String, serde_json::Value>,
}

pub fn parse_plugins(json: &str) -> Result<Vec<PluginInfo>> {
    serde_json::from_str(json).context("parsing `claude plugin list --json`")
}

/// Enabled plugins that declare the same MCP server name: (server, plugin ids).
pub fn duplicate_servers(plugins: &[PluginInfo]) -> Vec<(String, Vec<String>)> {
    let mut by: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for p in plugins.iter().filter(|p| p.enabled) {
        for s in p.mcp_servers.keys() {
            by.entry(s).or_default().push(p.id.clone());
        }
    }
    by.into_iter().filter(|(_, ids)| ids.len() > 1).map(|(s, ids)| (s.to_string(), ids)).collect()
}

pub const PLUGIN_ID: &str = "ilmarinen@ilmarinen";

/// The installed Claude Code plugin must exist, be enabled, and match the
/// version of this package's binaries. Duplicate server names are warned.
fn check_plugins() -> Result<bool> {
    println!("\nClaude Code plugins:");
    let Some(out) = util::sh("claude plugin list --json", None).filter(|o| o.ok) else {
        println!("  ✘ `claude plugin list --json` did not run");
        return Ok(false);
    };
    let plugins = parse_plugins(&out.text)?;
    let mut ok = true;
    match plugins.iter().find(|p| p.id == PLUGIN_ID) {
        None => {
            println!("  ✘ {PLUGIN_ID} is not installed; run `ilmarinen setup`");
            ok = false;
        }
        Some(p) if !p.enabled => {
            println!("  ✘ {PLUGIN_ID} is installed but disabled");
            ok = false;
        }
        Some(p) => {
            let want = Version::parse(&p.version);
            for bin in ["ilmarinen", "ilmarinen-sampo"] {
                let got = util::sh(&format!("{bin} --version"), None).and_then(|o| Version::find(&o.text));
                match (&want, got) {
                    (Some(w), Some(g)) if *w == g => {
                        println!("  ✔ {bin} {g} matches plugin {}", p.version)
                    }
                    (_, g) => {
                        let g = g.map_or("missing".to_string(), |v| v.to_string());
                        println!("  ✘ {bin} {g} does not match installed plugin {}", p.version);
                        ok = false;
                    }
                }
            }
        }
    }
    for (server, ids) in duplicate_servers(&plugins) {
        println!(
            "  ! MCP server `{server}` is declared by several enabled plugins: {} (doubles schema cost; ambiguous answers)",
            ids.join(", ")
        );
    }
    Ok(ok)
}

#[derive(Debug, PartialEq)]
pub enum Mcp {
    Connected,
    Failed,
    NotRegistered,
}

/// Parse one server's status out of `claude mcp list` / `opencode mcp list`.
pub fn mcp_line_status(listing: &str, server: &str) -> Mcp {
    let name = Regex::new(&format!(r"(^|[\s:/]){}(\s|:|$)", regex::escape(server))).unwrap();
    for line in listing.lines().filter(|l| name.is_match(l)) {
        let l = line.to_lowercase();
        if l.contains("failed") || l.contains("error") || l.contains("needs auth") {
            return Mcp::Failed;
        }
        if l.contains("connected") {
            return Mcp::Connected;
        }
    }
    Mcp::NotRegistered
}

pub const BM_PROJECT: &str = "agent-memory";

fn check_personal() -> Result<bool> {
    println!("\nPer-machine state:");
    let home = util::home()?;
    let mem = crate::setup::memory_dir()?;
    let mut ok = true;
    let mut check = |label: &str, good: bool| {
        println!("  {} {label}", if good { "✔" } else { "✘" });
        ok &= good;
    };
    check("config.toml", repos::config_path()?.is_file());
    check("repos.toml", repos::repos_path()?.is_file());
    check("memory project (git, pinned.md)", mem.join(".git").is_dir() && mem.join("pinned.md").is_file());
    check(&format!("Basic Memory project {BM_PROJECT:?}"), bm_project_exists(&home, &mem));
    if !ok {
        println!("  run `ilmarinen setup` to create these");
    }
    Ok(ok)
}

/// Read Basic Memory's config directly (offline) and look for our project.
pub fn bm_project_exists(home: &std::path::Path, mem: &std::path::Path) -> bool {
    let Ok(text) = std::fs::read_to_string(home.join(".basic-memory/config.json")) else {
        return false;
    };
    bm_config_has(&text, BM_PROJECT, mem)
}

pub fn bm_config_has(config_json: &str, name: &str, path: &std::path::Path) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(config_json) else {
        return false;
    };
    let Some(projects) = v.get("projects") else {
        return false;
    };
    let path_matches = |p: &serde_json::Value| {
        let p = p.as_str().or_else(|| p.get("path").and_then(|x| x.as_str()));
        p.is_some_and(|p| std::path::Path::new(p) == path)
    };
    match projects {
        serde_json::Value::Object(m) => m.get(name).is_some_and(path_matches),
        serde_json::Value::Array(a) => {
            a.iter().any(|e| e.get("name").and_then(|n| n.as_str()) == Some(name) && path_matches(e))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn mcp_listing_parse() {
        let claude = "Checking MCP server health...\n\nserena: uvx serena start-mcp-server - ✔ Connected\nplugin:ilmarinen:sampo: ilmarinen sampo serve - ✘ Failed to connect\ncontext7: https://mcp.context7.com/mcp (HTTP) - ! Needs authentication\n";
        assert_eq!(mcp_line_status(claude, "serena"), Mcp::Connected);
        assert_eq!(mcp_line_status(claude, "sampo"), Mcp::Failed);
        assert_eq!(mcp_line_status(claude, "context7"), Mcp::Failed);
        assert_eq!(mcp_line_status(claude, "playwright"), Mcp::NotRegistered);
        // Another plugin's same-named server must not count as ours.
        let two =
            "plugin:ilmarinen:context7: npx - ✘ Failed to connect\nplugin:context7:context7: https://x - ✔ Connected\n";
        assert_eq!(mcp_line_status(two, "plugin:ilmarinen:context7"), Mcp::Failed);
        let oc = "┌  MCP Servers\n│\n●  ✓ serena connected\n│      uvx serena\n●  ✗ basic-memory failed\n";
        assert_eq!(mcp_line_status(oc, "serena"), Mcp::Connected);
        assert_eq!(mcp_line_status(oc, "basic-memory"), Mcp::Failed);
        // "memory" must not match "basic-memory".
        assert_eq!(mcp_line_status(oc, "memory"), Mcp::NotRegistered);
    }

    #[test]
    fn duplicate_servers_across_enabled_plugins() {
        let json = r#"[
          {"id":"ilmarinen@ilmarinen","version":"0.1.0","enabled":true,"mcpServers":{"context7":{},"serena":{}}},
          {"id":"context7@claude-plugins-official","version":"1.0.0","enabled":true,"mcpServers":{"context7":{}}},
          {"id":"off@x","version":"1","enabled":false,"mcpServers":{"serena":{}}},
          {"id":"bare@x"}
        ]"#;
        let dups = duplicate_servers(&parse_plugins(json).unwrap());
        assert_eq!(
            dups,
            vec![(
                "context7".to_string(),
                vec!["ilmarinen@ilmarinen".to_string(), "context7@claude-plugins-official".to_string()]
            )]
        );
    }

    #[test]
    fn plugin_manifest_version_matches_crate() {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../../plugin/.claude-plugin/plugin.json")).unwrap();
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn telemetry_confirmation() {
        let out = "crash-reports: disabled (source: persistent state)\nusage-stats: disabled\n";
        assert!(confirmed(out, None, Some(r":\s*enabled")));
        assert!(!confirmed("usage-stats: enabled", None, Some(r":\s*enabled")));
        assert!(confirmed("metrics off", Some("off"), None));
        assert!(!confirmed("metrics on", Some("off"), None));
    }

    #[test]
    fn basic_memory_config_shapes() {
        let p = Path::new("/srv/m/memory");
        assert!(bm_config_has(r#"{"projects":{"agent-memory":"/srv/m/memory"}}"#, "agent-memory", p));
        assert!(bm_config_has(r#"{"projects":{"agent-memory":{"path":"/srv/m/memory"}}}"#, "agent-memory", p));
        assert!(bm_config_has(r#"{"projects":[{"name":"agent-memory","path":"/srv/m/memory"}]}"#, "agent-memory", p));
        assert!(!bm_config_has(r#"{"projects":{"agent-memory":"/elsewhere"}}"#, "agent-memory", p));
        assert!(!bm_config_has("not json", "agent-memory", p));
    }

    #[test]
    fn pins_cover_every_mise_key_in_tools() {
        let pins = mise_pins().unwrap();
        for t in manifest::parse(&assets::tools_toml()).unwrap() {
            if let Some(k) = &t.mise {
                assert!(pins.contains_key(k), "{} names mise key {k} not pinned in mise.toml", t.name);
            }
        }
    }
}
