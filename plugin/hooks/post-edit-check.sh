#!/bin/sh
# Ilmarinen PostToolUse check (Edit, Write): run `just check <lang>` for the
# language of the edited file. Exit 2 shows the failure to the agent.
set -u
input=$(cat)
f=$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')
[ -n "$f" ] || exit 0
case "$f" in
  *.rs) lang=rust ;;
  *.ts|*.tsx|*.mts|*.cts|*.js|*.jsx|*.mjs|*.cjs|*.vue|*.svelte) lang=ts ;;
  *.py|*.pyi) lang=python ;;
  */spec/*.md) lang=spec ;;
  *) exit 0 ;;
esac
dir=$(dirname "$f")
[ -d "$dir" ] || exit 0
root=$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null) || exit 0
command -v just >/dev/null 2>&1 || exit 0
[ -f "$root/justfile" ] || [ -f "$root/Justfile" ] || [ -f "$root/.justfile" ] || exit 0
out=$(cd "$root" && just check "$lang" 2>&1) && exit 0
{
  echo "ilmarinen: \`just check $lang\` failed after editing $f:"
  printf '%s\n' "$out" | tail -n 40
} >&2
exit 2
