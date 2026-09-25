//! Sampo: the cross-repo index (decision 0005). Walks the checkouts in
//! `repos.toml`, parses SpecScore frontmatter, header lines and headings into
//! `$HOME/.cache/ilmarinen/sampo.db` (SQLite + FTS5), and answers four
//! read-only queries: search, get, related, stale. An Artifact: rebuilt on
//! every server start, never authoritative.

use crate::{repos, util};
use anyhow::{Context, Result};
use regex::Regex;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub fn db_path() -> Result<PathBuf> {
    Ok(util::home()?.join(".cache/ilmarinen/sampo.db"))
}

const SCHEMA: &str = "
DROP TABLE IF EXISTS artifact; DROP TABLE IF EXISTS link; DROP TABLE IF EXISTS fts;
CREATE TABLE artifact(id TEXT PRIMARY KEY, repo TEXT, kind TEXT, status TEXT, title TEXT,
  summary TEXT, path TEXT, date TEXT, promotes_to TEXT);
CREATE TABLE link(src TEXT, rel TEXT, dst TEXT);
CREATE VIRTUAL TABLE fts USING fts5(id UNINDEXED, title, body);";

/// One parsed Blueprint or Rune.
#[derive(Debug, Default, PartialEq)]
pub struct Doc {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub title: String,
    pub summary: String,
    pub date: String,
    pub links: Vec<(String, String)>, // (rel, target id)
}

static HEADER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\*\*([A-Za-z ]+):\*\*\s*(.*)$").unwrap());
static SOURCE_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(//|#|--|[/*]|%|;)\s*(specscore:(implements|verifies|references)\s+|specscore:|https://specscore\.org/)(\S*)").unwrap()
});

/// Parse one spec file. `rel` is relative to `spec/` (e.g. `features/login/README.md`).
pub fn parse(repo: &str, rel: &str, text: &str) -> Option<Doc> {
    let (kind, slug) = kind_slug(rel)?;
    let header = |name: &str| {
        HEADER.captures_iter(text).find(|c| &c[1] == name).map(|c| c[2].trim().to_string()).filter(|v| v != "—")
    };
    let front = text.strip_prefix("---\n").and_then(|b| b.split_once("\n---")).map(|(f, _)| f);
    let status = front
        .and_then(|f| f.lines().find_map(|l| l.strip_prefix("status:")).map(|s| s.trim().to_string()))
        .or_else(|| header("Status"))
        .unwrap_or_default();
    let title = text
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(|t| t.split_once(": ").map_or(t, |(_, b)| b).to_string())
        .unwrap_or_default();
    let section = match kind {
        "feature" => "Summary",
        "decision" => "Decision",
        "lesson" => "Lesson",
        _ => "Instructions",
    };
    let summary = header("Statement").or_else(|| first_paragraph(text, section)).unwrap_or_default();
    let mut d = Doc {
        id: format!("{repo}:{kind}/{slug}"),
        kind: kind.into(),
        status,
        title,
        summary: words(&summary, 30),
        date: header("Date").unwrap_or_default(),
        links: vec![],
    };
    let target = |k: &str, v: &str| format!("{repo}:{k}/{}", v.trim());
    for (field, rel, k) in [("Supersedes", "supersedes", kind), ("Superseded By", "superseded_by", kind)] {
        if let Some(v) = header(field) {
            d.links.push((rel.into(), target(k, &v)));
        }
    }
    if let Some(v) = header("Promotes To") {
        d.links.push(("promotes_to".into(), target("rule", v.trim_start_matches("rule:"))));
    }
    for s in header("Sources").iter().flat_map(|v| v.split(',').map(str::trim).map(String::from).collect::<Vec<_>>()) {
        if let Some((k, v)) = s.split_once(':') {
            d.links.push(("source".into(), target(k, v)));
        }
    }
    if kind == "decision"
        && let Some(sec) = section_body(text, "Affected Features")
    {
        for l in sec.lines().filter_map(|l| l.trim().strip_prefix("- ")) {
            d.links.push(("affects".into(), target("feature", l.trim_matches('`'))));
        }
    }
    Some(d)
}

fn kind_slug(rel: &str) -> Option<(&'static str, String)> {
    let parts: Vec<&str> = rel.split('/').collect();
    let (kind, rest) = parts.split_first()?;
    let stem = |s: &str| s.trim_end_matches(".md").to_string();
    match (*kind, rest) {
        ("features", [.., "README.md"]) if rest.len() > 1 => Some(("feature", rest[..rest.len() - 1].join("/"))),
        ("decisions", [f]) | ("decisions", ["archived", f]) if *f != "README.md" && f.ends_with(".md") => {
            Some(("decision", stem(f)))
        }
        ("rules", [s, "README.md"]) => Some(("rule", s.to_string())),
        ("lessons", [s, "README.md"]) => Some(("lesson", s.to_string())),
        _ => None,
    }
}

fn section_body<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let start = text.find(&format!("\n## {heading}\n"))? + heading.len() + 5;
    let rest = &text[start..];
    Some(&rest[..rest.find("\n## ").unwrap_or(rest.len())])
}

fn first_paragraph(text: &str, heading: &str) -> Option<String> {
    let body = section_body(text, heading)?;
    let p =
        body.split("\n\n").map(str::trim).find(|p| !p.is_empty() && !p.starts_with('#') && !p.starts_with("<!--"))?;
    Some(p.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// At most `n` words (≈ 40 tokens for n = 30).
fn words(s: &str, n: usize) -> String {
    let w: Vec<&str> = s.split_whitespace().collect();
    if w.len() <= n { w.join(" ") } else { format!("{} …", w[..n].join(" ")) }
}

/// A Source Reference target → artifact id, relative to `repo`.
pub fn ref_target(repo: &str, t: &str) -> Option<String> {
    let t = t.split('#').next()?.trim_end_matches('/');
    let (repo, path) =
        if let Some(r) = t.strip_prefix("specscore://").or_else(|| t.strip_prefix("https://specscore.org/")) {
            let p: Vec<&str> = r.splitn(4, '/').collect();
            (p.get(..3)?.join("/"), p.get(3)?.to_string())
        } else {
            (repo.to_string(), t.to_string())
        };
    let path = path.strip_prefix("spec/").map(String::from).unwrap_or(path);
    let (kind, slug) = match path.split_once('/')? {
        ("feature" | "features", s) => ("feature", s),
        ("decisions", s) => ("decision", s.trim_end_matches(".md")),
        ("rules", s) => ("rule", s),
        ("lessons", s) => ("lesson", s),
        _ => return None,
    };
    Some(format!("{repo}:{kind}/{}", slug.trim_end_matches("/README.md")))
}

pub fn index(db: &Path, repos: &[repos::Repo]) -> Result<usize> {
    util::write(db, [])?;
    let mut c = Connection::open(db)?;
    c.execute_batch(SCHEMA)?;
    let tx = c.transaction()?;
    let mut n = 0;
    for r in repos {
        let listed =
            util::capture("git", &["ls-files", "-co", "--exclude-standard"], Some(&r.path)).unwrap_or_default();
        for rel in listed.lines().filter_map(|f| f.strip_prefix("spec/")).filter(|f| f.ends_with(".md")) {
            let f = r.path.join("spec").join(rel);
            let (Ok(text), path) = (std::fs::read_to_string(&f), f.to_string_lossy()) else { continue };
            let Some(d) = parse(&r.id, rel, &text) else { continue };
            tx.execute(
                "INSERT OR REPLACE INTO artifact VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    d.id,
                    r.id,
                    d.kind,
                    d.status,
                    d.title,
                    d.summary,
                    path,
                    d.date,
                    d.links.iter().find(|l| l.0 == "promotes_to").map(|l| &l.1)
                ],
            )?;
            tx.execute("INSERT INTO fts VALUES (?1,?2,?3)", params![d.id, d.title, text])?;
            for (rel, dst) in &d.links {
                tx.execute("INSERT INTO link VALUES (?1,?2,?3)", params![d.id, rel, dst])?;
            }
            n += 1;
        }
        for f in listed.lines().filter(|f| !f.starts_with("spec/") && !f.starts_with("artifacts/")) {
            let Ok(text) = std::fs::read_to_string(r.path.join(f)) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                let Some(c) = SOURCE_REF.captures(line) else {
                    continue;
                };
                let verb = c.get(3).map_or("references", |m| m.as_str());
                if let Some(dst) = ref_target(&r.id, &c[4]) {
                    tx.execute(
                        "INSERT INTO link VALUES (?1,?2,?3)",
                        params![format!("{}:code/{f}:{}", r.id, i + 1), verb, dst],
                    )?;
                }
            }
        }
    }
    tx.commit()?;
    Ok(n)
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    Ok(json!({"id": r.get::<_, String>(0)?, "repo": r.get::<_, String>(1)?, "kind": r.get::<_, String>(2)?,
              "status": r.get::<_, String>(3)?, "summary": r.get::<_, String>(4)?, "path": r.get::<_, String>(5)?}))
}

const COLS: &str = "a.id, a.repo, a.kind, a.status, a.summary, a.path";

/// Full-text search; every term must match (quoted, so FTS syntax is inert),
/// falling back to any term when nothing matches all.
pub fn search(c: &Connection, query: &str, repo: Option<&str>, kind: Option<&str>, k: usize) -> Result<Vec<Value>> {
    let terms: Vec<String> = query.split_whitespace().map(|t| format!("\"{}\"", t.replace('"', ""))).collect();
    if terms.is_empty() {
        return Ok(vec![]);
    }
    let sql = format!(
        "SELECT {COLS} FROM fts JOIN artifact a ON a.id = fts.id WHERE fts MATCH ?1 AND (?2 IS NULL OR a.repo = ?2) \
         AND (?3 IS NULL OR a.kind = ?3) ORDER BY bm25(fts) LIMIT ?4"
    );
    let mut st = c.prepare(&sql)?;
    for q in [terms.join(" "), terms.join(" OR ")] {
        let hits: Vec<Value> = st.query_map(params![q, repo, kind, k as i64], row)?.collect::<rusqlite::Result<_>>()?;
        if !hits.is_empty() {
            return Ok(hits);
        }
    }
    Ok(vec![])
}

pub fn get(c: &Connection, id: &str) -> Result<Value> {
    let mut v = c
        .query_row(&format!("SELECT {COLS} FROM artifact a WHERE a.id = ?1"), [id], row)
        .with_context(|| format!("no artifact {id}"))?;
    let text = std::fs::read_to_string(v["path"].as_str().unwrap_or_default()).unwrap_or_default();
    v["text"] = json!(text);
    Ok(v)
}

/// Links in both directions: supersedes/superseded_by, affects, promotes_to,
/// source, and Source References from code (implements/verifies/references).
pub fn related(c: &Connection, id: &str) -> Result<Vec<Value>> {
    let mut st = c.prepare(
        "SELECT dst, rel, 'out' FROM link WHERE src = ?1 UNION SELECT src, rel, 'in' FROM link WHERE dst = ?1 ORDER BY 3, 2, 1",
    )?;
    let out = st.query_map([id], |r| {
        Ok(json!({"id": r.get::<_, String>(0)?, "rel": r.get::<_, String>(1)?, "direction": r.get::<_, String>(2)?}))
    })?;
    Ok(out.collect::<rusqlite::Result<_>>()?)
}

/// Lessons still Recorded six weeks after their date with no Promotes To, and
/// links whose target artifact does not exist.
pub fn stale(c: &Connection, today: &str) -> Result<Vec<Value>> {
    let cutoff: String = c.query_row("SELECT date(?1, '-42 days')", [today], |r| r.get(0))?;
    let mut out = Vec::new();
    let mut st = c.prepare(&format!(
        "SELECT {COLS} FROM artifact a WHERE a.kind = 'lesson' AND a.status = 'Recorded' AND a.promotes_to IS NULL AND a.date <> '' AND a.date < ?1"
    ))?;
    for r in st.query_map([&cutoff], row)? {
        let mut v = r?;
        v["why"] = json!(format!("Recorded before {cutoff} and not promoted"));
        out.push(v);
    }
    let mut st = c.prepare("SELECT src, rel, dst FROM link WHERE dst NOT IN (SELECT id FROM artifact) ORDER BY 1")?;
    for r in st.query_map([], |r| Ok(json!({"id": r.get::<_, String>(0)?, "why": format!("{} {} does not exist", r.get::<_, String>(1)?, r.get::<_, String>(2)?)})))? {
        out.push(r?);
    }
    Ok(out)
}
