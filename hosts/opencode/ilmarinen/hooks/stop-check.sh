#!/bin/sh
# Ilmarinen Stop check, once per turn: run `just check <lang>` for each
# language with uncommitted changes. Exit 2 hands the failure back to the agent
# (Claude continues the turn); never re-blocks when stop_hook_active is set.
# Skips a tree it already passed (fingerprint in .git/ilmarinen-stop-check).
set -u
input=$(cat)
cwd=$(printf '%s' "$input" | jq -r '.cwd // empty'); [ -n "$cwd" ] || cwd=$(pwd)
. "${0%/*}/active.sh"; ilm_active "$cwd" || exit 0
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$root" || exit 0
command -v just >/dev/null 2>&1 && { [ -f justfile ] || [ -f Justfile ] || [ -f .justfile ]; } || exit 0
files=$(git status --porcelain --untracked-files=all | cut -c4- | sed 's/.* -> //')
langs=$(printf '%s\n' "$files" | awk '
  /\.rs$/ {print "rust"} /\.(ts|tsx|mts|cts|js|jsx|mjs|cjs|vue|svelte)$/ {print "ts"}
  /\.pyi?$/ {print "python"} /^spec\/.*\.md$/ {print "spec"}' | sort -u)
[ -n "$langs" ] || exit 0
state=$(git rev-parse --git-path ilmarinen-stop-check)
sum=$({ printf '%s\n' "$files"; git diff HEAD 2>/dev/null || git diff --cached; git ls-files -o --exclude-standard -z | xargs -0 cat 2>/dev/null; } | cksum)
[ "$(cat "$state" 2>/dev/null)" = "$sum" ] && exit 0
out=
for l in $langs; do
  r=$(just check "$l" 2>&1) || out="$out
\`just check $l\` failed:
$(printf '%s\n' "$r" | tail -n 30)"
done
if [ -z "$out" ]; then printf '%s\n' "$sum" > "$state"; exit 0; fi
[ "$(printf '%s' "$input" | jq -r '.stop_hook_active // false')" = true ] && exit 0
printf 'ilmarinen: checks failed on uncommitted changes:%s\n' "$out" >&2
exit 2
