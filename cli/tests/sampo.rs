//! Sampo: parsing, indexing two repos, the four queries, and the MCP protocol.

use ilmarinen::{repos::Repo, sampo};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn git_init(root: &Path) {
    assert!(Command::new("git").args(["init", "-q"]).current_dir(root).status().unwrap().success());
}

const FEATURE: &str = "---\nformat: https://specscore.md/feature-specification\nstatus: Approved\n---\n\n# Feature: Login\n\n**Status:** Approved\n\n## Summary\n\nUsers sign in with a passkey.\n\n## Behavior\n\n#### REQ: passkey\n\nThe service SHALL accept passkeys.\n";
const OLD: &str = "# Decision: Sessions in Redis\n\n**Status:** Superseded\n**Date:** 2026-01-02\n**Supersedes:** —\n**Superseded By:** 0002-sessions-in-postgres\n\n## Decision\n\nKeep sessions in Redis.\n\n## Affected Features\n\n- login\n";
const NEW: &str = "# Decision: Sessions in Postgres\n\n**Status:** Approved\n**Date:** 2026-03-04\n**Supersedes:** 0001-sessions-in-redis\n**Superseded By:** —\n\n## Decision\n\nKeep sessions in Postgres for durability.\n\n## Affected Features\n\nNone at this time.\n";
const LESSON: &str = "---\nstatus: Recorded\n---\n# Lesson: Pin the pre-release\n\n**Status:** Recorded\n**Date:** 2026-01-01\n\n## Lesson\n\nPin every pre-release dependency explicitly.\n";

fn fixture() -> (tempfile::TempDir, Vec<Repo>) {
    let t = tempfile::TempDir::new().unwrap();
    let a = t.path().join("a");
    let b = t.path().join("b");
    for r in [&a, &b] {
        std::fs::create_dir_all(r).unwrap();
        git_init(r);
    }
    write(&a, "spec/features/login/README.md", FEATURE);
    write(&a, "spec/features/README.md", "# Features index\n");
    write(&a, "spec/decisions/archived/0001-sessions-in-redis.md", OLD);
    write(&a, "spec/decisions/0002-sessions-in-postgres.md", NEW);
    write(&a, "spec/lessons/pin-pre-release/README.md", LESSON);
    write(&a, "src/login.rs", "// specscore:implements feature/login#req:passkey\nfn login() {}\n");
    write(&b, "web/login.ts", "// specscore:references specscore://github.com/acme/a/feature/login#req:passkey\n");
    write(&b, "web/gone.ts", "// specscore:verifies feature/missing#ac:x\n");
    let repos =
        vec![Repo { id: "github.com/acme/a".into(), path: a }, Repo { id: "github.com/acme/b".into(), path: b }];
    (t, repos)
}

#[test]
fn parses_kinds_and_headers() {
    let d = sampo::parse("r", "features/login/README.md", FEATURE).unwrap();
    assert_eq!(
        (d.id.as_str(), d.kind.as_str(), d.status.as_str(), d.title.as_str()),
        ("r:feature/login", "feature", "Approved", "Login")
    );
    assert_eq!(d.summary, "Users sign in with a passkey.");
    let n = sampo::parse("r", "decisions/0002-sessions-in-postgres.md", NEW).unwrap();
    assert_eq!(n.links, vec![("supersedes".to_string(), "r:decision/0001-sessions-in-redis".to_string())]);
    let o = sampo::parse("r", "decisions/archived/0001-sessions-in-redis.md", OLD).unwrap();
    assert!(o.links.contains(&("affects".into(), "r:feature/login".into())));
    assert!(sampo::parse("r", "features/README.md", "# index").is_none(), "indexes are not artifacts");
    assert!(sampo::parse("r", "plans/x/README.md", "# plan").is_none());
}

#[test]
fn reference_targets() {
    assert_eq!(sampo::ref_target("h/o/r", "feature/login#req:x").unwrap(), "h/o/r:feature/login");
    assert_eq!(sampo::ref_target("h/o/r", "specscore://h/o/other/feature/a/b#ac:y").unwrap(), "h/o/other:feature/a/b");
    assert_eq!(
        sampo::ref_target("h/o/r", "https://specscore.org/h/o/x/spec/features/login#req:z").unwrap(),
        "h/o/x:feature/login"
    );
    assert!(sampo::ref_target("h/o/r", "docs/readme.md").is_none());
}

#[test]
fn index_and_query() {
    let (t, repos) = fixture();
    let db = t.path().join("sampo.db");
    assert_eq!(sampo::index(&db, &repos).unwrap(), 4);
    let c = Connection::open(&db).unwrap();

    let hits = sampo::search(&c, "passkey", None, None, 5).unwrap();
    assert_eq!(hits[0]["id"], "github.com/acme/a:feature/login");
    assert_eq!(hits[0]["kind"], "feature");
    assert!(
        sampo::search(&c, "sessions durability", None, Some("decision"), 5).unwrap()[0]["id"]
            .as_str()
            .unwrap()
            .ends_with("0002-sessions-in-postgres")
    );
    assert!(sampo::search(&c, "passkey", Some("github.com/acme/b"), None, 5).unwrap().is_empty());
    assert!(!sampo::search(&c, "passkey OR NEAR( \"", None, None, 5).unwrap().is_empty(), "FTS syntax is inert");

    assert!(
        sampo::get(&c, "github.com/acme/a:feature/login").unwrap()["text"].as_str().unwrap().contains("REQ: passkey")
    );
    assert!(sampo::get(&c, "nope").is_err());

    let rel = sampo::related(&c, "github.com/acme/a:feature/login").unwrap();
    let has = |id: &str, r: &str| rel.iter().any(|v| v["id"] == id && v["rel"] == r);
    assert!(has("github.com/acme/a:code/src/login.rs:1", "implements"));
    assert!(has("github.com/acme/b:code/web/login.ts:1", "references"), "cross-repo reference");
    assert!(has("github.com/acme/a:decision/0001-sessions-in-redis", "affects"));
    let rel = sampo::related(&c, "github.com/acme/a:decision/0001-sessions-in-redis").unwrap();
    assert!(rel.iter().any(|v| v["rel"] == "superseded_by" && v["direction"] == "out"));
    assert!(rel.iter().any(|v| v["rel"] == "supersedes" && v["direction"] == "in"));

    let stale = sampo::stale(&c, "2026-09-24").unwrap();
    assert!(stale.iter().any(|v| v["id"] == "github.com/acme/a:lesson/pin-pre-release"));
    assert!(stale.iter().any(|v| v["why"].as_str().unwrap().contains("feature/missing")));
    assert!(sampo::stale(&c, "2026-01-20").unwrap().iter().all(|v| v["kind"] != "lesson"), "not yet six weeks");
}

#[test]
fn mcp_protocol() {
    let (t, repos) = fixture();
    let cfg = t.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();
    let toml: String =
        repos.iter().map(|r| format!("[[repo]]\nid = \"{}\"\npath = \"{}\"\n\n", r.id, r.path.display())).collect();
    std::fs::write(cfg.join("repos.toml"), toml).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ilmarinen-sampo"))
        .arg("serve")
        .env("HOME", t.path())
        .env("ILMARINEN_CONFIG", &cfg)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let reqs = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"query":"passkey"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"bogus"}),
    ];
    let mut stdin = child.stdin.take().unwrap();
    for r in &reqs {
        writeln!(stdin, "{r}").unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let lines: Vec<Value> =
        String::from_utf8(out.stdout).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.len(), 5, "the notification gets no response");
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "sampo");
    let names: Vec<&str> =
        lines[1]["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["search", "get", "related", "stale"]);
    let hits: Value = serde_json::from_str(lines[2]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(hits[0]["id"], "github.com/acme/a:feature/login");
    assert_eq!(lines[3]["result"]["isError"], true);
    assert_eq!(lines[4]["error"]["code"], -32601);
    assert!(t.path().join(".cache/ilmarinen/sampo.db").exists());
}
