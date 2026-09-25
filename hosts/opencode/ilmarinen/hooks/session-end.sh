#!/bin/sh
# Ilmarinen SessionEnd: write a three-line handoff note, synchronously, to
# ~/.cache/ilmarinen/handoff/<repo>.md: where work stopped, what is next, what
# is waiting. SessionStart flushes it into Beads. Claude Code gives SessionEnd
# hooks 1.5 s, so this makes one `bd list` call (~0.8 s) and no other.
set -u
cwd=$(jq -r '.cwd // empty' 2>/dev/null || true); [ -n "$cwd" ] || cwd=$(pwd)
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -d "$root/.beads" ] && command -v bd >/dev/null 2>&1 || exit 0
dir=${XDG_CACHE_HOME:-$HOME/.cache}/ilmarinen/handoff
json=$(cd "$root" && BD_DISABLE_METRICS=1 bd list --status open,in_progress --json 2>/dev/null) || exit 0
note=$(printf '%s' "$json" | jq -r '
  def few(f; none): [.[] | select(f)] | sort_by(.priority) | .[:3] | map("\(.id) \(.title)") | if length == 0 then none else join("; ") end;
  "Stopped at: " + few(.status == "in_progress"; "nothing in progress"),
  "Next: " + few(.status == "open" and (.dependency_count // 0) == 0; "nothing ready"),
  "Waiting on dependencies: " + few(.status == "open" and (.dependency_count // 0) > 0; "nothing")' 2>/dev/null) || exit 0
mkdir -p "$dir" && printf '%s\n' "$note" > "$dir/$(basename "$root").md"
exit 0
