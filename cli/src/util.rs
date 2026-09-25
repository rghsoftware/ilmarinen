//! Small shared helpers: paths, subprocesses, prompts, managed text blocks.

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn home() -> Result<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).context("HOME is not set")
}

/// `$ILMARINEN_CONFIG`, else `$HOME/.config/ilmarinen` (the hooks read the same).
pub fn config_dir() -> Result<PathBuf> {
    match std::env::var_os("ILMARINEN_CONFIG") {
        Some(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => Ok(home()?.join(".config/ilmarinen")),
    }
}

pub struct Output {
    pub ok: bool,
    pub text: String,
}

/// Run a command line through `sh -c`, capturing stdout+stderr. `None` if it
/// could not be spawned at all.
pub fn sh(cmd: &str, cwd: Option<&Path>) -> Option<Output> {
    let mut c = Command::new("sh");
    c.arg("-c").arg(cmd).stdin(Stdio::null());
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let out = c.output().ok()?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Some(Output { ok: out.status.success(), text })
}

/// Like [`sh`] but with the given environment variables removed.
pub fn sh_without(cmd: &str, unset: &[&str]) -> Option<Output> {
    let mut c = Command::new("sh");
    c.arg("-c").arg(cmd).stdin(Stdio::null());
    for k in unset {
        c.env_remove(k);
    }
    let out = c.output().ok()?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Some(Output { ok: out.status.success(), text })
}

/// Expand a leading `~/` to `$HOME/`.
pub fn expand_home(p: &str) -> Result<PathBuf> {
    Ok(match p.strip_prefix("~/") {
        Some(rest) => home()?.join(rest),
        None => PathBuf::from(p),
    })
}

/// Run a program with arguments, inheriting stdio; fail on non-zero exit.
pub fn run(prog: &str, args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut c = Command::new(prog);
    c.args(args);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let st = c.status().with_context(|| format!("running {prog}"))?;
    if !st.success() {
        bail!("`{prog} {}` exited with {st}", args.join(" "));
    }
    Ok(())
}

/// Run a program with arguments, capturing output; fail on non-zero exit.
pub fn capture(prog: &str, args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut c = Command::new(prog);
    c.args(args).stdin(Stdio::null());
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let out = c.output().with_context(|| format!("running {prog}"))?;
    if !out.status.success() {
        bail!("`{prog} {}` failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn which(prog: &str) -> bool {
    std::env::var_os("PATH").map(|p| std::env::split_paths(&p).any(|d| d.join(prog).is_file())).unwrap_or(false)
}

pub fn interactive() -> bool {
    std::io::stdin().is_terminal()
}

/// Ask a yes/no question. Non-interactive stdin always answers no: consent is
/// never assumed.
pub fn confirm(question: &str) -> bool {
    if !interactive() {
        println!("{question} [y/N] n (non-interactive)");
        return false;
    }
    print!("{question} [y/N] ");
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).ok();
    matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

const BEGIN: &str = "<!-- ilmarinen:begin (managed by `ilmarinen setup`; edits here are overwritten) -->";
const END: &str = "<!-- ilmarinen:end -->";

/// Insert or replace the ilmarinen-managed block in `existing`, leaving every
/// other line untouched.
pub fn upsert_block(existing: &str, body: &str) -> String {
    let block = format!("{BEGIN}\n{}\n{END}\n", body.trim_end());
    if let (Some(b), Some(e)) = (existing.find("<!-- ilmarinen:begin"), existing.find(END))
        && b < e
    {
        let after = &existing[e + END.len()..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        return format!("{}{block}{after}", &existing[..b]);
    }
    if existing.is_empty() {
        return block;
    }
    let sep = if existing.ends_with("\n\n") {
        ""
    } else if existing.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{existing}{sep}{block}")
}

/// Write a file, creating parent directories.
pub fn write(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).with_context(|| format!("creating {}", p.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_appends_then_replaces_in_place() {
        let user = "# My rules\n\n1. Be brief.\n";
        let once = upsert_block(user, "a\nb");
        assert!(once.starts_with(user));
        assert!(once.contains("a\nb\n<!-- ilmarinen:end -->"));
        let twice = upsert_block(&format!("{once}\n# Later\n"), "c");
        assert!(twice.starts_with(user));
        assert!(!twice.contains("\na\n"));
        assert!(twice.contains("c\n<!-- ilmarinen:end -->"));
        assert!(twice.ends_with("# Later\n"));
        assert_eq!(twice.matches("ilmarinen:begin").count(), 1);
    }

    #[test]
    fn block_into_empty_file() {
        assert!(upsert_block("", "x").starts_with("<!-- ilmarinen:begin"));
    }
}
