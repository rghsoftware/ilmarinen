#!/bin/sh
# Ilmarinen PreToolUse audit (Bash): never blocks. When a command bypasses or
# disables a git hook, append a `hook` event to the repo's scorecard
# (scripts/scorecard.sh, in repos stamped by `ilmarinen init`).
set -u
input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // empty')
cwd=$(printf '%s' "$input" | jq -r '.cwd // empty'); [ -n "$cwd" ] || cwd=$(pwd)
. "${0%/*}/active.sh"; ilm_active "$cwd" || exit 0
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -x "$root/scripts/scorecard.sh" ] || exit 0
log() { (cd "$root" && scripts/scorecard.sh hook "$1" "$2" "$3") >/dev/null 2>&1 || true; }
has() { printf '%s\n' "$cmd" | grep -Eq "$1"; }
S='[[:space:]]'
if has "(^|[;&|(]|$S)git$S[^;&|]*commit($S|\$)"; then
  has "$S(--no-verify|-[[:alpha:]]*n[[:alpha:]]*)($S|\$)" && log pre-commit bypassed no-verify
fi
has "(^|[;&|(]|$S)git$S[^;&|]*push$S[^;&|]*--no-verify" && log pre-push bypassed no-verify
has "git$S+-c$S*core\\.hooks[Pp]ath" && log git-hooks bypassed hooksPath-override
has "git$S+config$S[^;&|]*core\\.hooks[Pp]ath" && log git-hooks disabled hooksPath-changed
has "bd$S+hooks$S+uninstall" && log beads-hooks disabled uninstalled
exit 0
