#!/bin/sh
# Ilmarinen SessionStart: print the Beads handoff note and `bd ready` first,
# and log (and say) when the repo's leak guard is not wired. Stdout becomes
# session context in Claude Code, so keep it short.
set -u
cwd=$(jq -r '.cwd // empty' 2>/dev/null || true); [ -n "$cwd" ] || cwd=$(pwd)
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$root" || exit 0
export BD_DISABLE_METRICS=1
if [ -d .beads ] && command -v bd >/dev/null 2>&1; then
  note=$(bd recall ilmarinen-handoff 2>/dev/null || true)
  echo "## Ilmarinen: session start"
  if [ -n "$note" ]; then echo "Last session:"; printf '%s\n' "$note" | sed 's/^/  /'; fi
  echo "Ready (bd ready):"
  bd ready 2>/dev/null | grep -E '^[[:space:]]*[0-9]+\.|^[[:space:]]*[○◐●]' | head -n 10 | sed 's/^/  /'
fi
if [ -f .ilmarinen.version ]; then
  hook=$(git rev-parse --path-format=absolute --git-path hooks 2>/dev/null)/pre-commit
  if ! grep -q 'ILMARINEN LEAK GUARD' "$hook" 2>/dev/null; then
    echo "Warning: the leak guard is not in $hook; run \`ilmarinen init .\` to re-wire it."
    [ -x scripts/scorecard.sh ] && scripts/scorecard.sh hook leak-guard disabled not-wired >/dev/null 2>&1
  fi
fi
exit 0
