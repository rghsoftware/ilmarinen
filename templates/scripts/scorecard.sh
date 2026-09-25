#!/bin/sh
# Append one event to artifacts/scorecard.jsonl (an Artifact; `just retro` reads it).
#   scorecard.sh gate <gate> <feature> fired [--caught] [--tokens N]
#   scorecard.sh gate <gate> <feature> skipped --reason <word> [--tokens N]
#   scorecard.sh done <feature> <trivial|standard|major> [--tokens N]
#   scorecard.sh hook <hook> <disabled|bypassed> <detail-word>
# A skip without a one-word reason is refused: there is no silent skip path.
set -eu
die() { echo "scorecard: $*" >&2; exit 2; }
word() { printf '%s' "$1" | grep -Eqx '[A-Za-z0-9._-]+'; }
root=$(git rev-parse --show-toplevel); out=$root/artifacts/scorecard.jsonl
kind=${1:-}; [ $# -gt 0 ] && shift
caught=false reason= tokens=
case "$kind" in
  gate) [ $# -ge 3 ] || die "usage: gate <gate> <feature> fired|skipped [...]"
        gate=$1 feature=$2 status=$3; shift 3
        case "$gate" in intent|decision|plan|checks|review|merge) ;; *) die "unknown gate '$gate'" ;; esac
        case "$status" in fired|skipped) ;; *) die "status must be fired or skipped" ;; esac ;;
  done) [ $# -ge 2 ] || die "usage: done <feature> <depth>"; feature=$1 depth=$2; shift 2
        case "$depth" in trivial|standard|major) ;; *) die "depth must be trivial, standard or major" ;; esac ;;
  hook) [ $# -ge 3 ] || die "usage: hook <hook> <disabled|bypassed> <detail-word>"; hook=$1 event=$2 detail=$3; shift 3
        case "$event" in disabled|bypassed) ;; *) die "hook event must be disabled or bypassed" ;; esac
        word "$detail" || die "detail must be one word" ;;
  *) die "usage: scorecard.sh gate|done|hook ..." ;;
esac
while [ $# -gt 0 ]; do
  case "$1" in
    --caught) caught=true ;;
    --reason) [ $# -ge 2 ] || die "--reason needs a word"; reason=$2; shift ;;
    --tokens) [ $# -ge 2 ] || die "--tokens needs a number"; tokens=$2; shift ;;
    *) die "unknown option $1" ;;
  esac; shift
done
[ -z "$tokens" ] || printf '%s' "$tokens" | grep -Eqx '[0-9]+' || die "--tokens must be a whole number"
if [ "$kind" = gate ] && [ "$status" = skipped ]; then
  [ -n "$reason" ] || die "a skipped gate needs --reason <one word>"
  word "$reason" || die "the skip reason must be one word"
  [ "$caught" = false ] || die "a skipped gate cannot have caught something"
fi
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
case "$kind" in
  gate) line=$(jq -cn --arg ts "$ts" --arg g "$gate" --arg f "$feature" --arg s "$status" --argjson c "$caught" --arg r "$reason" --arg t "$tokens" \
          '{v:1,ts:$ts,kind:"gate",gate:$g,feature:$f,status:$s,caught:$c,reason:(if $r=="" then null else $r end),tokens:(if $t=="" then null else ($t|tonumber) end)}') ;;
  done) line=$(jq -cn --arg ts "$ts" --arg f "$feature" --arg d "$depth" --arg t "$tokens" \
          '{v:1,ts:$ts,kind:"done",feature:$f,depth:$d,tokens:(if $t=="" then null else ($t|tonumber) end)}') ;;
  hook) line=$(jq -cn --arg ts "$ts" --arg h "$hook" --arg e "$event" --arg d "$detail" '{v:1,ts:$ts,kind:"hook",hook:$h,event:$e,detail:$d}') ;;
esac
mkdir -p "$root/artifacts"; printf '%s\n' "$line" >> "$out"
