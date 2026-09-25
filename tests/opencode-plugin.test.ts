// Exercises hosts/opencode/plugins/ilmarinen.ts the way OpenCode calls it.
// Run: `bun test tests/opencode-plugin.test.ts` (part of `just test`).
import { expect, test } from "bun:test"
import { mkdtempSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { IlmarinenHooks } from "../hosts/opencode/plugins/ilmarinen.ts"

const repo = mkdtempSync(join(tmpdir(), "ilm-oc-"))
Bun.spawnSync(["git", "init", "-q", repo])
writeFileSync(join(repo, "justfile"), 'check lang="all":\n    @test "{{lang}}" != rust\n')
writeFileSync(join(repo, ".ilmarinen.version"), "0.1.2\n")
const hooks: any = await IlmarinenHooks({ directory: repo } as any)
const before = (tool: string, args: object) => hooks["tool.execute.before"]({ tool, sessionID: "s", callID: "c" }, { args })

test("blocks force push", async () => {
  await expect(before("bash", { command: "git push --force origin main" })).rejects.toThrow(/force/)
})
test("blocks reading .env", async () => {
  await expect(before("read", { filePath: join(repo, ".env") })).rejects.toThrow(/\.env/)
})
test("allows ordinary commands and reads", async () => {
  await before("bash", { command: "git status" })
  await before("read", { filePath: join(repo, "justfile") })
})
test("ignores tools it does not map", async () => {
  await before("webfetch", { url: "https://example.com" })
})
test("session.idle runs the Stop check; a failure is reported, not thrown", async () => {
  writeFileSync(join(repo, "x.rs"), "")
  const warned: string[] = []
  const warn = console.warn
  console.warn = (m: string) => { warned.push(m) }
  try { await hooks.event({ event: { type: "session.idle", properties: {} } }) } finally { console.warn = warn }
  expect(warned.join("\n")).toContain("just check rust")
  Bun.spawnSync(["rm", join(repo, "x.rs")])
})
test("dormant in a repo that is not initialized", async () => {
  const bare = mkdtempSync(join(tmpdir(), "ilm-oc-bare-"))
  Bun.spawnSync(["git", "init", "-q", bare])
  const dormant: any = await IlmarinenHooks({ directory: bare } as any)
  await dormant["tool.execute.before"]({ tool: "bash", sessionID: "s", callID: "c" }, { args: { command: "git push --force" } })
})
test("session.idle runs the handoff hook quietly; other events are ignored", async () => {
  await hooks.event({ event: { type: "session.idle", properties: {} } })
  await hooks.event({ event: { type: "file.edited", properties: {} } })
})
