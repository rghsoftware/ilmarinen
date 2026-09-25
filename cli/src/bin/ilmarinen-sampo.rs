//! `ilmarinen-sampo serve | index | --version`. `serve` rebuilds the index,
//! then speaks MCP (JSON-RPC over stdio) with four read-only tools. MCP tool
//! names cannot contain dots, so `sampo.search` is the `search` tool of the
//! `sampo` server (Claude Code shows it as `mcp__plugin_ilmarinen_sampo__search`).

use ilmarinen::{repos, sampo};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn build() -> anyhow::Result<Connection> {
    let db = sampo::db_path()?;
    let list = repos::load_repos(&repos::repos_path()?)?;
    let n = sampo::index(&db, &list.repos)?;
    eprintln!("sampo: indexed {n} artifact(s) from {} repo(s) into {}", list.repos.len(), db.display());
    Ok(Connection::open(db)?)
}

fn tools() -> Value {
    let id = json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]});
    json!([
        {"name": "search", "description": "Search Blueprints and Runes across all repos. Returns id, repo, kind, status, summary, path.",
         "inputSchema": {"type": "object", "properties": {
            "query": {"type": "string"}, "repo": {"type": "string", "description": "host/org/repo"},
            "kind": {"type": "string", "enum": ["feature", "decision", "rule", "lesson"]},
            "k": {"type": "integer", "default": 5}}, "required": ["query"]}},
        {"name": "get", "description": "One artifact by id, with its full text.", "inputSchema": id},
        {"name": "related", "description": "Artifacts and code linked to an id: supersedes, affects, promotes_to, source, implements, verifies, references.", "inputSchema": id},
        {"name": "stale", "description": "Lessons unpromoted after six weeks, and links to artifacts that do not exist.", "inputSchema": {"type": "object", "properties": {}}}
    ])
}

fn call(c: &Connection, name: &str, a: &Value) -> anyhow::Result<Value> {
    let s = |k: &str| a.get(k).and_then(Value::as_str);
    let id = || s("id").ok_or_else(|| anyhow::anyhow!("`id` is required"));
    Ok(match name {
        "search" => {
            let k = a.get("k").and_then(Value::as_u64).unwrap_or(5).clamp(1, 50) as usize;
            json!(sampo::search(c, s("query").unwrap_or_default(), s("repo"), s("kind"), k)?)
        }
        "get" => sampo::get(c, id()?)?,
        "related" => json!(sampo::related(c, id()?)?),
        "stale" => json!(sampo::stale(c, "now")?),
        other => anyhow::bail!("unknown tool {other}"),
    })
}

fn handle(c: &Connection, req: &Value) -> Option<Value> {
    let id = req.get("id")?.clone(); // notifications get no response
    let result = match req.get("method").and_then(Value::as_str).unwrap_or_default() {
        "initialize" => json!({
            "protocolVersion": req.pointer("/params/protocolVersion").cloned().unwrap_or(json!("2025-06-18")),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "sampo", "version": VERSION}}),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools()}),
        "tools/call" => {
            let name = req.pointer("/params/name").and_then(Value::as_str).unwrap_or_default();
            let args = req.pointer("/params/arguments").cloned().unwrap_or(json!({}));
            let (text, is_error) =
                call(c, name, &args).map_or_else(|e| (format!("{e:#}"), true), |v| (v.to_string(), false));
            json!({"content": [{"type": "text", "text": text}], "isError": is_error})
        }
        m => return Some(error(id, -32601, &format!("method not found: {m}"))),
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn serve(c: &Connection) -> anyhow::Result<()> {
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let resp = match serde_json::from_str::<Value>(&line) {
            Ok(req) => handle(c, &req),
            Err(_) if line.trim().is_empty() => None,
            Err(e) => Some(error(Value::Null, -32700, &e.to_string())),
        };
        if let Some(r) = resp {
            writeln!(out, "{r}")?;
            out.flush()?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let r = match std::env::args().nth(1).unwrap_or_default().as_str() {
        "--version" | "-V" => {
            println!("ilmarinen-sampo {VERSION}");
            Ok(())
        }
        "index" => build().map(drop),
        "serve" => build().and_then(|c| serve(&c)),
        _ => Err(anyhow::anyhow!("usage: ilmarinen-sampo serve | index | --version")),
    };
    if let Err(e) = r {
        eprintln!("error: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
