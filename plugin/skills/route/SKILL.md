---
name: route
description: Classify a request as trivial, standard or major before any work starts, and name the gates that apply. Use first on every new request, and again whenever scope grows.
---

# route

Classify the request before touching any file.

1. Look for the opt-in keyword (it is in `AGENTS.md` under Routing, and in the
   user instructions). If the request uses it, the depth is `major`.
2. Check impact. Search Sampo for related Blueprints and Runes first:
   `sampo.search("<key terms>")`, then `sampo.related(<id>)` on hits.
3. Pick the depth:

| Depth | When | Gates | Next skill |
|---|---|---|---|
| `trivial` | a fix or small change with no interface, data-model or cross-repo impact | merge | `forge` |
| `standard` | a bounded change inside one repo | plan, merge | `plan` |
| `major` | keyword used, or it touches an interface contract (API, CLI, file or wire format, MCP schema), a data model (schema, migration, persisted format), or more than one repo | intent+AC, decision, plan, merge | `blueprint` |

4. State it in one sentence and stop for the human:
   `Route: <depth>, because <reason>. Gates: <list>. Next: <skill>.`
   The human may override the depth.

Rules:
- Never escalate silently. If scope grows mid-task (a new interface, data model
  or repo), stop, say what changed, and re-route.
- `trivial` emits no Blueprint and no Beads issue.
- Never record what the developer owns (identity, hosts, network, paths,
  accounts, deployment targets) anywhere. If routing needs such a fact, run a
  command in this session and do not persist the result.
