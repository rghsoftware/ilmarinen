#!/bin/sh
# Ilmarinen PreToolUse guard (Bash, Read, Grep). Exit 2 blocks; stderr is the reason.
# Generic rules only, no infrastructure lists: git push --force*; rm -rf outside
# the worktree; reads of .env*; database or deploy commands whose target is not
# local (non-loopback DB URLs or hosts, non-local kubectl/helm contexts, cloud CLIs).
set -u
input=$(cat)
field() { printf '%s' "$input" | jq -r "$1 // empty"; }
deny() { printf 'ilmarinen: blocked: %s\n' "$1" >&2; exit 2; }
local_host() { case "$1" in localhost|::1|'[::1]'|0.0.0.0|/*) return 0 ;; esac; printf '%s' "$1" | grep -Eqx '127(\.[0-9]{1,3}){3}'; }
tool=$(field .tool_name)
cwd=$(field .cwd); [ -n "$cwd" ] || cwd=$(pwd)
case "$tool" in
  Read|Grep) p=$(field '.tool_input.file_path // .tool_input.path')
    case "${p##*/}" in .env|.env.*) deny "reading $p (.env* is never read)" ;; esac; exit 0 ;;
  Bash) cmd=$(field .tool_input.command) ;;
  *) exit 0 ;;
esac
has() { printf '%s\n' "$cmd" | grep -Eq "$1"; }
S='[[:space:]]'
has "(^|[;&|(]|$S)git$S[^;&|]*push($S|\$)" && has "$S(--force[^[:space:]]*|-[[:alpha:]]*f[[:alpha:]]*)($S|\$)" &&
  deny "git push --force (never force-push)"
has "(^|[[:space:]\"'=/<>])\\.env(\\.[[:alnum:]_.-]+)?(\$|[[:space:]\"';|&)>])" && deny "command touches a .env* file"
for u in $(printf '%s\n' "$cmd" | grep -Eo '(postgres(ql)?|mysql|mariadb|mongodb(\+srv)?|rediss?|surrealdb?|clickhouse|sqlserver)://[^[:space:]"'"'"']+'); do
  h=${u#*://}; h=${h##*@}; h=${h%%[/?]*}; h=${h%:[0-9]*}
  local_host "$h" || deny "database URL target is not local"
done

root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null || printf '%s' "$cwd")
set -f
printf '%s\n' "$cmd" | tr ';&|' '\n\n\n' | while IFS= read -r seg; do
  set -- $seg
  [ "${1:-}" = sudo ] && shift
  c=${1:-}; [ -n "$c" ] || continue; shift
  case "$c" in
    aws|gcloud|az|doctl|fly|flyctl|heroku|vercel|netlify|wrangler|eksctl|oci|hcloud|linode-cli|railway)
      deny "cloud CLI '$c' (deploys and cloud changes never run from an agent session)" ;;
    terraform|tofu|pulumi) case "${1:-}" in apply|destroy|import|up) deny "$c $1" ;; esac ;;
    kubectl|helm) ctx=; prev=
      for a; do case "$prev" in --context|--kube-context) ctx=$a ;; esac; case "$a" in --context=*|--kube-context=*) ctx=${a#*=} ;; esac; prev=$a; done
      [ -n "$ctx" ] || ctx=$(kubectl config current-context 2>/dev/null || true)
      case "$ctx" in kind-*|k3d-*|minikube|docker-desktop|rancher-desktop|orbstack|colima*) ;; *) deny "$c context is not a local cluster" ;; esac ;;
    psql|pg_dump|pg_restore|mysql|mysqldump|mariadb|mongosh|redis-cli|clickhouse-client|surreal|sqlcmd) prev=
      for a; do h=
        case "$prev" in -h|--host|--endpoint|-S) h=$a ;; esac
        case "$a" in --host=*|--endpoint=*) h=${a#*=} ;; -h?*) h=${a#-h} ;; esac
        h=${h#*://}; h=${h%%[/?]*}; h=${h%:[0-9]*}
        [ -z "$h" ] || local_host "$h" || deny "$c target is not local"; prev=$a
      done ;;
    rm) r=; f=; targets=
      for a; do case "$a" in --recursive) r=1 ;; --force) f=1 ;; --*) ;; -*) case "$a" in *[rR]*) r=1 ;; esac; case "$a" in *f*) f=1 ;; esac ;; *) targets="$targets $a" ;; esac; done
      [ -n "$r" ] && [ -n "$f" ] || continue
      for t in $targets; do
        case "$t" in '~'*|'$'*|*..*) deny "rm -rf $t (cannot prove it is inside the worktree)" ;; /*) abs=$t ;; *) abs=$cwd/$t ;; esac
        case "$abs" in "$root"/?*) ;; *) deny "rm -rf $t is outside the worktree" ;; esac
      done ;;
  esac
done || exit 2
exit 0
