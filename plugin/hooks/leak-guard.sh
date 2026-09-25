#!/bin/sh
# Ilmarinen leak guard: fail when lines being committed carry an identifying
# shape: an absolute path under /home or /Users, a private address (RFC 1918
# IPv4; IPv6 unique-local fc00::/7 or link-local fe80::/10), or any line of a
# .env* file (except *.example). Optional extra fixed strings, case-insensitive:
# ~/.config/ilmarinen/leak-patterns.txt, which starts empty and which the
# package never writes. Runs first in the git pre-commit
# hook, and as a Claude Code PreToolUse hook on `git commit` (stdin JSON gives
# the cwd). Exit 2 blocks in both hosts; git treats any non-zero as failure.
set -u
if [ ! -t 0 ]; then
  input=$(cat)
  cwd=$(printf '%s' "$input" | jq -r '.cwd // empty' 2>/dev/null || true)
  [ -n "${cwd:-}" ] && cd "$cwd"
fi
# Dormant unless initialized and not off (decision 0014); standalone copy of active.sh.
d=$(pwd); while [ ! -f "$d/.ilmarinen.version" ]; do [ "$d" = / ] && exit 0; d=$(dirname "$d"); done
[ -f "$d/.ilmarinen.off" ] && exit 0
extras=${ILMARINEN_CONFIG:-$HOME/.config/ilmarinen}/leak-patterns.txt

# Added lines of the staged diff, as "<file>: <line>".
added=$(git diff --cached -U0 --no-color --no-ext-diff |
  awk '/^\+\+\+ /{f=substr($0,7); next} /^\+/{print f ": " substr($0,2)}')
[ -n "$added" ] || exit 0

home='(^|[^[:alnum:]_.~-])/(home|Users)/[[:alnum:]_.-]+'
rfc1918='(^|[^0-9.])(10\.[0-9]{1,3}|172\.(1[6-9]|2[0-9]|3[01])|192\.168)\.[0-9]{1,3}\.[0-9]{1,3}($|[^0-9.])'
ula='(^|[^[:xdigit:]:])(f[cd][[:xdigit:]]{2}|fe[89ab][[:xdigit:]]):[[:xdigit:]]{0,4}:[[:xdigit:]:]*[[:xdigit:]]'
hits=$(printf '%s\n' "$added" | grep -Ei -e "$home" -e "$rfc1918" -e "$ula")
envs=$(printf '%s\n' "$added" | awk '{n=$1; sub(/:$/, "", n); sub(/.*\//, "", n)} n ~ /^\.env/ && n !~ /\.example$/')
extra=
if [ -s "$extras" ]; then
  tmp=$(mktemp) || exit 2
  grep -v -e '^[[:space:]]*#' -e '^[[:space:]]*$' "$extras" > "$tmp" || true
  [ -s "$tmp" ] && extra=$(printf '%s\n' "$added" | grep -F -i -f "$tmp")
  rm -f "$tmp"
fi
all=$(printf '%s\n%s\n%s\n' "$hits" "$envs" "$extra" | grep -v '^$' | sort -u | cut -c1-200)
[ -z "$all" ] && exit 0
{
  echo "ilmarinen: leak guard: staged lines carry identifying information"
  echo "(home-directory paths, private IPv4/IPv6 addresses, .env contents, or listed extras):"
  printf '%s\n' "$all" | sed 's/^/  /'
  echo "Remove it from the change. Nothing about you, your machines or your network belongs in a repo."
} >&2
exit 2
