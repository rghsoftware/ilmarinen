#!/bin/sh
# Ilmarinen SessionEnd: write a three-line handoff into Beads
# (`bd remember --key ilmarinen-handoff`): where work stopped, what is next,
# what is blocked. Claude Code gives SessionEnd hooks 1.5 s and a plugin
# cannot raise that, so the Beads calls run detached and the hook returns at once.
set -u
cwd=$(jq -r '.cwd // empty' 2>/dev/null || true); [ -n "$cwd" ] || cwd=$(pwd)
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -d "$root/.beads" ] && command -v bd >/dev/null 2>&1 || exit 0
list() { jq -r --arg none "$2" 'if length == 0 then $none else (.[:3] | map("\(.id) \(.title)") | join("; ")) end' 2>/dev/null <<EOF2 || echo "$2"
$1
EOF2
}
(
  cd "$root" || exit 0
  export BD_DISABLE_METRICS=1
  stopped=$(list "$(bd list --status in_progress --json 2>/dev/null)" "nothing in progress")
  next=$(list "$(bd ready --json 2>/dev/null)" "nothing ready")
  blocked=$(list "$(bd blocked --json 2>/dev/null)" "nothing blocked")
  bd remember "Stopped at: $stopped
Next: $next
Blocked: $blocked" --key ilmarinen-handoff >/dev/null 2>&1
) </dev/null >/dev/null 2>&1 &
exit 0
