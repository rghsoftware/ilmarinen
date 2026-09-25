//! `tools.toml`: the tool manifest `doctor` checks against.

use anyhow::{Context, Result, bail};
use regex::Regex;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    tool: BTreeMap<String, Tool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    #[serde(skip)]
    pub name: String,
    pub detect: String,
    pub min: String,
    /// Key in `mise.toml`; absent when mise cannot manage the tool.
    pub mise: Option<String>,
    pub install: String,
    pub required: bool,
    pub role: String,
    /// `dev` (developer machines only), `ci` (CI only) or `all`.
    pub scope: String,
    pub telemetry: Option<Telemetry>,
}

impl Tool {
    /// Whether `doctor` checks this tool in CI mode (`--skip-mcp`).
    pub fn in_ci(&self) -> bool {
        self.scope != "dev"
    }
}

/// How a tool that phones home is opted out, and how the opt-out is confirmed.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Telemetry {
    /// What it sends, for the report.
    pub sends: String,
    /// Env vars that opt out per process; `just`, hooks and CI set these.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Persistent opt-out command `setup` runs, if the tool has one.
    pub disable: Option<String>,
    /// Command whose output proves the opt-out is in effect.
    pub confirm: Option<String>,
    /// Regex the confirm output must match.
    pub confirmed_if: Option<String>,
    /// Regex the confirm output must not match.
    pub confirmed_unless: Option<String>,
    /// JSON settings file whose `env` object carries the opt-out (`~` expands).
    pub json_env: Option<String>,
}

pub fn parse(text: &str) -> Result<Vec<Tool>> {
    let raw: Raw = toml::from_str(text).context("parsing tools.toml")?;
    let mut tools = Vec::with_capacity(raw.tool.len());
    for (name, mut t) in raw.tool {
        if t.detect.trim().is_empty() {
            bail!("tools.toml: [tool.{name}] has an empty detect command");
        }
        if !["dev", "ci", "all"].contains(&t.scope.as_str()) {
            bail!("tools.toml: [tool.{name}] scope must be dev, ci or all (got {:?})", t.scope);
        }
        Version::parse(&t.min)
            .with_context(|| format!("tools.toml: [tool.{name}] min {:?} is not a version", t.min))?;
        if let Some(tel) = &t.telemetry {
            if tel.confirm.is_some() == (tel.confirmed_if.is_none() && tel.confirmed_unless.is_none()) {
                bail!(
                    "tools.toml: [tool.{name}.telemetry] confirm needs confirmed_if or confirmed_unless, and vice versa"
                );
            }
            if tel.json_env.is_some() && (tel.confirm.is_some() || tel.env.is_empty()) {
                bail!("tools.toml: [tool.{name}.telemetry] json_env needs env and excludes confirm");
            }
            for re in [&tel.confirmed_if, &tel.confirmed_unless].into_iter().flatten() {
                Regex::new(re).with_context(|| format!("tools.toml: [tool.{name}.telemetry] bad regex {re:?}"))?;
            }
        }
        t.name = name;
        tools.push(t);
    }
    Ok(tools)
}

/// A dotted numeric version; missing components compare as zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version(pub Vec<u64>);

static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+(?:\.\d+)+").unwrap());

impl Version {
    pub fn parse(s: &str) -> Option<Version> {
        let s = s.trim().trim_start_matches('v');
        if s.is_empty() {
            return None;
        }
        s.split('.').map(|p| p.parse().ok()).collect::<Option<Vec<u64>>>().map(Version)
    }

    /// First dotted version in arbitrary command output (`just 1.58.0`, `node v26.8.2`).
    pub fn find(output: &str) -> Option<Version> {
        VERSION_RE.find(output).and_then(|m| Version::parse(m.as_str()))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let n = self.0.len().max(other.0.len());
        (0..n)
            .map(|i| self.0.get(i).unwrap_or(&0).cmp(other.0.get(i).unwrap_or(&0)))
            .find(|o| o.is_ne())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self.0.iter().map(u64::to_string).collect();
        f.write_str(&parts.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: &str = r#"
[tool.just]
detect = "just --version"
min = "1.58.0"
mise = "just"
install = "mise use -g just@1.58.0"
required = true
role = "gates"
scope = "all"
"#;

    #[test]
    fn parses_minimal_tool() {
        let t = &parse(MIN).unwrap()[0];
        assert_eq!(t.name, "just");
        assert_eq!(t.mise.as_deref(), Some("just"));
        assert!(t.required && t.telemetry.is_none());
    }

    #[test]
    fn parses_telemetry_block() {
        let text = format!(
            "{MIN}\n[tool.just.telemetry]\nsends = \"usage\"\nenv = {{ X_TELEMETRY = \"0\" }}\ndisable = \"just off\"\nconfirm = \"just status\"\nconfirmed_unless = \"enabled\"\n"
        );
        let tel = parse(&text).unwrap()[0].telemetry.clone().unwrap();
        assert_eq!(tel.env["X_TELEMETRY"], "0");
        assert_eq!(tel.confirmed_unless.as_deref(), Some("enabled"));
    }

    #[test]
    fn json_env_rules() {
        let ok =
            format!("{MIN}\n[tool.just.telemetry]\nsends = \"x\"\nenv = {{ A = \"1\" }}\njson_env = \"~/s.json\"\n");
        assert!(parse(&ok).is_ok());
        let no_env = format!("{MIN}\n[tool.just.telemetry]\nsends = \"x\"\njson_env = \"~/s.json\"\n");
        assert!(parse(&no_env).is_err());
    }

    #[test]
    fn rejects_confirm_without_matcher() {
        let text = format!("{MIN}\n[tool.just.telemetry]\nsends = \"x\"\nconfirm = \"just status\"\n");
        assert!(parse(&text).is_err());
    }

    #[test]
    fn rejects_unknown_field_and_bad_min() {
        assert!(parse(&MIN.replace("role =", "rolee =")).is_err());
        assert!(parse(&MIN.replace("1.58.0\"\nmise", "latest\"\nmise")).is_err());
        assert!(parse(&MIN.replace("scope = \"all\"", "scope = \"prod\"")).is_err());
    }

    #[test]
    fn repo_manifest_parses() {
        let tools = parse(include_str!("../../tools.toml")).unwrap();
        assert!(tools.iter().any(|t| t.name == "bd" && t.required));
        for host in ["claude", "opencode"] {
            assert!(!tools.iter().find(|t| t.name == host).unwrap().in_ci(), "{host} is dev-only");
        }
        for t in &tools {
            assert!(!t.install.is_empty(), "{} has no install hint", t.name);
        }
    }

    #[test]
    fn finds_versions_in_real_outputs() {
        let cases = [
            ("git version 2.43.0", "2.43.0"),
            ("node v26.8.2", "26.8.2"),
            ("2.1.281 (Claude Code)", "2.1.281"),
            ("bd version 1.3.0 (f45b249ce: HEAD@f45b249ce6b4)", "1.3.0"),
            ("codegrapher 0.13.2 (0d0f0d041c9ad1b0) 2026-09-20T10:00:10Z", "0.13.2"),
            ("mise 2026.9.10 linux-x64 (2026-09-16)", "2026.9.10"),
            ("uv 0.12.18 (x86_64-unknown-linux-gnu)", "0.12.18"),
        ];
        for (out, want) in cases {
            assert_eq!(Version::find(out).unwrap().to_string(), want, "{out}");
        }
        assert!(Version::find("command not found").is_none());
    }

    #[test]
    fn compares_versions() {
        let v = |s| Version::parse(s).unwrap();
        assert!(v("1.58.0") >= v("1.58"));
        assert!(v("1.10.0") > v("1.9.9"));
        assert!(v("2026.9.10") > v("2026.9.0"));
        assert!(v("0.12.15") < v("0.12.18"));
        assert_eq!(Version::parse("v1.2"), Some(Version(vec![1, 2])));
        assert!(Version::parse("1.x").is_none());
    }
}
