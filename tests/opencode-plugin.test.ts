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
test("post-edit failure is appended to the tool output", async () => {
  const f = join(repo, "x.rs"); writeFileSync(f, "")
  const output = { title: "", output: "edited", metadata: {} }
  await hooks["tool.execute.after"]({ tool: "edit", sessionID: "s", callID: "c", args: { filePath: f } }, output)
  expect(output.output).toContain("just check rust")
  const ok = { title: "", output: "edited", metadata: {} }
  const py = join(repo, "y.py"); writeFileSync(py, "")
  await hooks["tool.execute.after"]({ tool: "write", sessionID: "s", callID: "c", args: { filePath: py } }, ok)
  expect(ok.output).toBe("edited")
})
test("session.idle runs the handoff hook quietly; other events are ignored", async () => {
  await hooks.event({ event: { type: "session.idle", properties: {} } })
  await hooks.event({ event: { type: "file.edited", properties: {} } })
})
