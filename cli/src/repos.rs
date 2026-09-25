//! Per-machine state under `$HOME/.config/ilmarinen/` (or `$ILMARINEN_CONFIG`):
//! `repos.toml` (checkouts Sampo indexes) and `config.toml` (`opt_in_keyword`).
//! Nothing here describes the developer or their machines (decision 0013).

use crate::util;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const REPOS_HEADER: &str = "\
# Repos known to Ilmarinen on this machine. Personal: never commit this file.
# Sampo indexes these checkouts; scripts/trace.sh resolves specscore:// references
# against them. `ilmarinen init` appends entries.
";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Repos {
    #[serde(default, rename = "repo")]
    pub repos: Vec<Repo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Repo {
    /// `host/org/repo`, as used in `specscore://` references.
    pub id: String,
    pub path: PathBuf,
}

pub fn repos_path() -> Result<PathBuf> {
    Ok(util::config_dir()?.join("repos.toml"))
}

pub fn load_repos(path: &Path) -> Result<Repos> {
    if !path.exists() {
        return Ok(Repos::default());
    }
    let text = std::fs::read_to_string(path)?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Append a repo unless its path or id is already listed. Returns whether it
/// was added. Appends text so comments in the file survive.
pub fn add_repo(path: &Path, repo: &Repo) -> Result<bool> {
    let existing = load_repos(path)?;
    if existing.repos.iter().any(|r| r.path == repo.path || r.id == repo.id) {
        return Ok(false);
    }
    let mut text = std::fs::read_to_string(path).unwrap_or_else(|_| REPOS_HEADER.to_string());
    if !text.ends_with('\n') {
        text.push('\n');
    }
    let entry = toml::to_string(&Repos { repos: vec![repo.clone()] })?;
    text.push('\n');
    text.push_str(&entry);
    util::write(path, text)?;
    Ok(true)
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    /// The word that opts a request into `major` depth.
    pub opt_in_keyword: Option<String>,
}

pub fn config_path() -> Result<PathBuf> {
    Ok(util::config_dir()?.join("config.toml"))
}

pub fn load_config() -> Result<Config> {
    let p = config_path()?;
    if !p.exists() {
        return Ok(Config::default());
    }
    toml::from_str(&std::fs::read_to_string(&p)?).with_context(|| format!("parsing {}", p.display()))
}

pub fn save_config(c: &Config) -> Result<()> {
    util::write(&config_path()?, toml::to_string(c)?)
}

/// Parse `host/org/repo` from a git remote URL (scp-style, ssh://, https://).
pub fn remote_id(url: &str) -> Option<(String, String, String)> {
    let url = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let rest = if let Some(r) = url.split_once("://").map(|(_, r)| r) {
        r.rsplit_once('@').map_or(r, |(_, h)| h).to_string()
    } else {
        let (user_host, path) = url.split_once(':')?;
        let host = user_host.rsplit_once('@').map_or(user_host, |(_, h)| h);
        format!("{host}/{path}")
    };
    let mut parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 3 {
        return None;
    }
    // Strip a port from the host (ssh://git@host:2222/org/repo).
    let host = parts[0].split(':').next()?.to_string();
    let repo = parts.pop()?.to_string();
    let org = parts[1..].join("/");
    Some((host, org, repo))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parses_remote_forms() {
        let want = Some(("github.com".into(), "rghsoftware".into(), "ilmarinen".into()));
        assert_eq!(remote_id("git@github.com:rghsoftware/ilmarinen.git"), want);
        assert_eq!(remote_id("https://github.com/rghsoftware/ilmarinen"), want);
        assert_eq!(remote_id("ssh://git@github.com:22/rghsoftware/ilmarinen.git"), want);
        assert_eq!(
            remote_id("https://gitlab.example.com/group/sub/proj.git"),
            Some(("gitlab.example.com".into(), "group/sub".into(), "proj".into()))
        );
        assert_eq!(remote_id("not a url"), None);
    }

    #[test]
    fn add_repo_is_idempotent_and_keeps_comments() {
        let t = TempDir::new().unwrap();
        let p = t.path().join("repos.toml");
        let r = Repo { id: "github.com/o/a".into(), path: "/src/a".into() };
        assert!(add_repo(&p, &r).unwrap());
        assert!(!add_repo(&p, &r).unwrap());
        let r2 = Repo { id: "github.com/o/b".into(), path: "/src/b".into() };
        assert!(add_repo(&p, &r2).unwrap());
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.starts_with("# Repos known"));
        assert_eq!(load_repos(&p).unwrap().repos, vec![r, r2]);
    }
}
