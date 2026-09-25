#!/bin/sh
# Tests for plugin/hooks/*.sh. Run under dash (POSIX): `just test`.
# Identifying-looking test data is assembled at runtime so this file itself
# never trips the leak guard.
set -u
H=$(cd "$(dirname "$0")/../plugin/hooks" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
check() { # name expected-exit actual-exit
  if [ "$2" = "$3" ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $1 (want $2, got $3)"; fi
}
repo=$T/repo; git init -q "$repo"; git -C "$repo" config user.email t@example.invalid; git -C "$repo" config user.name t
root=$(git -C "$repo" rev-parse --show-toplevel)
export ILMARINEN_CONFIG=$T/cfg; mkdir -p "$ILMARINEN_CONFIG"
HOMEDIR=$(printf '/%s/%s' home someone)   # an absolute home path
PRIV=$(printf '%s.%s.%s.%s' 192 168 4 20)  # an RFC 1918 address
PUB=$(printf '%s.%s.%s.%s' 203 0 113 7)    # TEST-NET-3, not private
ULA=$(printf '%s:%s::%s' fd12 3456 7)      # IPv6 unique-local
LL=$(printf '%s::%s' fe80 1)               # IPv6 link-local
DOC6=$(printf '%s:%s::%s' 2001 db8 1)      # IPv6 documentation range, not private

bash_in() { jq -n --arg c "$1" --arg d "${2:-$root}" '{tool_name:"Bash",cwd:$d,tool_input:{command:$c}}'; }
deny() { bash_in "$2" | sh "$H/pre-tool-deny.sh" 2>/dev/null; check "$1: $2" "$3" $?; }
deny force1 "git push --force origin main" 2
deny force2 "git push -f" 2
deny force3 "git push --force-with-lease" 2
deny force4 "git -C x push -uf origin" 2
deny push-ok "git push origin main" 0
deny fetch-f "git fetch -f && git status" 0
deny env1 "cat .env" 2
deny env2 "source ./config/.env.local" 2
deny env3 "cat src/env.rs .envrc-notes" 0
deny rm-in "rm -rf build target/debug" 0
deny rm-abs-in "rm -rf $root/dist" 0
deny rm-tmp "rm -rf /tmp/x" 2
deny rm-home "rm -rf ~/x" 2
deny rm-up "rm -rf ../other" 2
deny rm-root "rm -fr /" 2
deny rm-self "rm -rf $root" 2
deny rm-chain "cd src && rm -rf /etc" 2
deny rm-sudo "sudo rm -r -f /var" 2
deny rm-long "rm --recursive --force /opt" 2
deny rm-no-f "rm -r /tmp/x" 0
deny db-url-local "psql postgres://app@localhost:5432/app -c 'select 1'" 0
deny db-url-loop "DATABASE_URL=postgresql://u@127.0.0.1/db just test" 0
deny db-url-remote "psql postgres://app@db.example.invalid/app" 2
deny db-url-ip "mongosh mongodb+srv://u@$PUB/x" 2
deny db-host-remote "psql -h db.example.invalid -U app" 2
deny db-host-joined "mysql -hdb.example.invalid app" 2
deny db-host-local "redis-cli -h 127.0.0.1 ping" 0
deny db-socket "psql -h /var/run/postgresql app" 0
deny db-any-local "redis-cli -h 0.0.0.0 ping" 0
deny db-loop-other "psql postgres://app@127.0.0.2/app" 0
deny db-127-name "psql -h 127.0.0.1.attacker.example app" 2
deny db-v6-loop "psql postgres://app@[::1]:5432/app" 0
deny db-v6-remote "psql postgres://app@[$ULA]:5432/app" 2
deny db-v6-host "psql -h $ULA app" 2
deny db-surreal "surreal sql --endpoint ws://db.example.invalid:8000" 2
deny db-surreal-local "surreal sql --endpoint ws://localhost:58000" 0
deny cloud1 "aws s3 ls" 2
deny cloud2 "cd infra && gcloud run deploy" 2
deny tf-apply "terraform apply -auto-approve" 2
deny tf-plan "terraform plan" 0
deny kube-remote "kubectl --context=shared-cluster get pods" 2
deny kube-local "kubectl --context kind-dev get pods" 0
deny helm-local "helm install x ./chart --kube-context k3d-test" 0
for p in /x/.env /x/.env.production; do
  jq -n --arg p "$p" '{tool_name:"Read",tool_input:{file_path:$p}}' | sh "$H/pre-tool-deny.sh" 2>/dev/null; check "read $p" 2 $?
done
jq -n '{tool_name:"Read",tool_input:{file_path:"/x/envy.txt"}}' | sh "$H/pre-tool-deny.sh"; check "read envy" 0 $?
jq -n '{tool_name:"Grep",tool_input:{pattern:"K",path:"/x/.env"}}' | sh "$H/pre-tool-deny.sh" 2>/dev/null; check "grep .env" 2 $?
grep -Eq '(^|[^._[:alnum:]])[a-z0-9-]+\.(com|net|org|io|dev|internal|local)([^[:alnum:]]|$)' "$H/pre-tool-deny.sh"; check "no hostnames in the deny hook" 1 $?

# leak-guard: generic shapes; extras file optional and empty by default.
guard() { (cd "$repo" && sh "$H/leak-guard.sh" </dev/null 2>/dev/null); check "leak: $1" "$2" $?; }
stage() { printf '%s\n' "$2" > "$repo/$1"; git -C "$repo" add "$1"; }
reset() { git -C "$repo" reset -q; }
stage a.txt "ok line"; guard clean 0; reset
stage b.txt "log dir $HOMEDIR/logs"; guard "home path" 2; reset
stage c.txt "$(printf 'path /%s/%s/x' Users someone)"; guard "macOS home path" 2; reset
stage d.txt "db at $PRIV"; guard "private address" 2; reset
stage e.txt "docs at $PUB and version 10.2.3"; guard "public address and version" 0; reset
stage u.txt "db at [$ULA]:5432"; guard "IPv6 unique-local" 2; reset
stage l.txt "iface $LL%eth0"; guard "IPv6 link-local" 2; reset
stage v.txt "example $DOC6, hash fd3a9c, time 12:30:45, css #fcfcfc"; guard "IPv6 public, hex and times" 0; reset
stage w.txt "listen on 127.0.0.1:8080, 0.0.0.0, [::1] or localhost"; guard "loopback allowed" 0; reset
stage r.txt "ranges fc00::/7 and fe80::/10 in prose"; guard "IPv6 range notation" 0; reset
stage .env.local "TOKEN=x"; guard ".env contents" 2; reset
stage .env.example "TOKEN="; guard ".env.example allowed" 0; reset
stage f.txt "cache under ~/.cache/tool"; guard "tilde path allowed" 0; reset
printf '# extras\nsecret-host-42\n' > "$ILMARINEN_CONFIG/leak-patterns.txt"
stage g.txt "ssh SECRET-HOST-42"; guard "optional extra (case-insensitive)" 2
jq -n --arg d "$root" '{cwd:$d}' | sh "$H/leak-guard.sh" 2>/dev/null; check "leak via hook JSON cwd" 2 $?
reset; rm "$ILMARINEN_CONFIG/leak-patterns.txt"
stage g.txt "ssh SECRET-HOST-42"; guard "no extras file" 0
git -C "$repo" commit -qm base

# post-edit-check
cat > "$repo/justfile" <<'EOF'
check lang="all":
    @test "{{lang}}" != rust
EOF
edit() { jq -n --arg f "$1" '{tool_name:"Edit",tool_input:{file_path:$f}}' | sh "$H/post-edit-check.sh" 2>/dev/null; check "edit $1" "$2" $?; }
touch "$repo/x.rs" "$repo/y.py" "$repo/z.txt"
edit "$repo/x.rs" 2
edit "$repo/y.py" 0
edit "$repo/z.txt" 0
edit "$T/not-a-repo.rs" 0

# Inside a linked worktree: the worktree is the boundary; config comes from
# $ILMARINEN_CONFIG / $HOME, never from the checkout.
wt=$repo/.worktrees/w1
git -C "$repo" worktree add -q "$wt" -b w1
wroot=$(git -C "$wt" rev-parse --show-toplevel)
wdeny() { bash_in "$2" "$wroot" | sh "$H/pre-tool-deny.sh" 2>/dev/null; check "worktree $1: $2" "$3" $?; }
wdeny rm-in "rm -rf build" 0
wdeny rm-main "rm -rf $root/src" 2
wdeny rm-up "rm -rf ../w2" 2
wdeny db "psql -h db.example.invalid" 2
printf 'x %s\n' "$PRIV" > "$wt/c.txt"; git -C "$wt" add c.txt
jq -n --arg d "$wroot" '{cwd:$d}' | sh "$H/leak-guard.sh" 2>/dev/null; check "worktree leak hit" 2 $?
git -C "$wt" reset -q c.txt
(cd "$wt" && sh "$H/leak-guard.sh" </dev/null); check "worktree leak clean" 0 $?
cp "$repo/justfile" "$wt/justfile"; touch "$wt/x.rs"
jq -n --arg f "$wt/x.rs" '{tool_name:"Edit",tool_input:{file_path:$f}}' | sh "$H/post-edit-check.sh" 2>/dev/null; check "worktree post-edit uses worktree justfile" 2 $?

# hook-audit: logs bypasses into a stamped repo's scorecard, never blocks.
mkdir -p "$repo/scripts"; cp "$H/../../templates/scripts/scorecard.sh" "$repo/scripts/"
audit() { bash_in "$1" | sh "$H/hook-audit.sh"; check "audit exit: $1" 0 $?; }
audit "git commit --no-verify -m x"
audit "git commit -anm wip"
audit "git -c core.hooksPath=/dev/null commit -m x"
audit "git commit -m 'no n flag here'"
audit "git status"
n=$(wc -l < "$repo/artifacts/scorecard.jsonl" 2>/dev/null || echo 0); check "audit logged 3 bypasses" 3 "$n"
jq -e 'select(.kind=="hook" and .detail=="hooksPath-override")' "$repo/artifacts/scorecard.jsonl" >/dev/null; check "audit hooksPath event" 0 $?

# session hooks without Beads: silent, fast, exit 0.
jq -n --arg d "$root" '{cwd:$d}' | sh "$H/session-end.sh"; check "session-end no beads" 0 $?
out=$(jq -n --arg d "$root" '{cwd:$d}' | sh "$H/session-start.sh"); check "session-start no beads exit" 0 $?
check "session-start no beads silent" "" "$out"
touch "$repo/.ilmarinen.version"
out=$(jq -n --arg d "$root" '{cwd:$d}' | sh "$H/session-start.sh")
case "$out" in *"leak guard is not in"*) r=0 ;; *) r=1 ;; esac; check "session-start warns unwired guard" 0 $r
jq -e 'select(.detail=="not-wired")' "$repo/artifacts/scorecard.jsonl" >/dev/null; check "unwired guard logged" 0 $?

# Handoff: SessionEnd writes the note synchronously within Claude Code's 1.5 s
# budget; SessionStart flushes it into Beads, removes the file, prints it first.
if command -v bd >/dev/null 2>&1; then
  export XDG_CACHE_HOME=$T/xdg BD_DISABLE_METRICS=1
  b=$T/beadsrepo; git init -q "$b"; git -C "$b" config user.email t@example.invalid; git -C "$b" config user.name t
  (cd "$b" && bd init --non-interactive --skip-agents --skip-hooks -q -p hb >/dev/null 2>&1)
  (cd "$b" && bd create "first task" -t task -p 1 >/dev/null 2>&1)
  ms() { t=$(date +%s%N 2>/dev/null); case "$t" in *N) echo $(( $(date +%s) * 1000 )) ;; *) echo $(( t / 1000000 )) ;; esac; }
  s=$(ms); jq -n --arg d "$b" '{cwd:$d}' | sh "$H/session-end.sh"; took=$(( $(ms) - s ))
  f=$XDG_CACHE_HOME/ilmarinen/handoff/$(basename "$b").md
  [ "$(grep -c '^Stopped at: \|^Next: \|^Waiting on dependencies: ' "$f" 2>/dev/null)" = 3 ]; check "session-end writes three lines" 0 $?
  grep -q '^Next: hb-.* first task' "$f"; check "session-end names the next issue" 0 $?
  [ "$took" -lt 1500 ]; check "session-end fits the 1.5 s budget (${took} ms)" 0 $?
  out=$(jq -n --arg d "$b" '{cwd:$d}' | sh "$H/session-start.sh")
  [ ! -e "$f" ]; check "session-start flushes the file" 0 $?
  (cd "$b" && bd recall ilmarinen-handoff 2>/dev/null | grep -q 'first task'); check "note is in Beads" 0 $?
  case "$out" in *"Last session:"*"Next: hb-"*"Ready (bd ready):"*"first task"*) r=0 ;; *) r=1 ;; esac; check "session-start prints note then bd ready" 0 $r
  ! grep -rq '&$\|) *&' "$H/session-end.sh"; check "session-end has no background writes" 0 $?
fi

echo "hooks: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
