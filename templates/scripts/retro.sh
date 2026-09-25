#!/bin/sh
# `just retro`: print the trial scorecard from artifacts/scorecard.jsonl using
# the criteria in scorecard.toml. Once `features` features are finished, print
# KEEP or REVISIT, naming each failing criterion.
set -eu
root=$(git rev-parse --show-toplevel); cd "$root"
log=artifacts/scorecard.jsonl; conf=scorecard.toml
[ -f "$conf" ] || { echo "retro: $conf not found" >&2; exit 1; }
[ -s "$log" ] || { echo "retro: no events yet in $log"; exit 0; }
num() { awk -F= -v k="$1" '$1 ~ "^[[:space:]]*"k"[[:space:]]*$" {gsub(/[^0-9]/, "", $2); print $2; exit}' "$conf"; }
arr() { awk -F= -v k="$1" '$1 ~ "^[[:space:]]*"k"[[:space:]]*$" {print $2; exit}' "$conf" | tr -d ' []'; }
jq -s -r \
  --argjson n "$(num features)" --argjson minc "$(num min_catches)" \
  --argjson maxh "$(num max_hook_disables)" --argjson maxs "$(num max_unreasoned_skips)" \
  --arg trivial "$(arr trivial)" --arg standard "$(arr standard)" --arg major "$(arr major)" '
  def req($d): ({trivial: $trivial, standard: $standard, major: $major}[$d] | gsub("\""; "") | split(","));
  . as $ev
  | ([$ev[] | select(.kind == "done")] | reverse | unique_by(.feature) | sort_by(.ts)) as $done
  | ($done | .[-$n:]) as $win
  | ($win | map(.feature)) as $wf
  | [$ev[] | select(.kind == "gate" and (.feature as $f | $wf | index($f)))] as $g
  | [$ev[] | select(.kind == "hook")] as $hooks
  | ([$g[] | select(.caught)] | length) as $catches
  | ([$win[] | . as $d | req($d.depth)[] | . as $gate
      | select(([$g[] | select(.feature == $d.feature and .gate == $gate)] | length) == 0)
      | {gate: $gate}]
     + [$g[] | select(.status == "skipped" and .reason == null) | {gate}]) as $unreasoned
  | ([$unreasoned | group_by(.gate)[] | {gate: .[0].gate, n: length}]) as $ucount
  | ([$ev[] | .tokens // empty] | add // null) as $tok
  | "Ilmarinen trial scorecard (\($ev | length) events, \($done | length) finished features; judging the last \($win | length))",
    "",
    "gate      fired  skipped  caught  skip reasons",
    ( ["intent","decision","plan","checks","review","merge"][] as $gate
      | [$g[] | select(.gate == $gate)] as $x
      | "\($gate | . + "          " | .[0:9]) \([$x[] | select(.status == "fired")] | length | tostring | . + "      " | .[0:6]) \([$x[] | select(.status == "skipped")] | length | tostring | . + "        " | .[0:8]) \([$x[] | select(.caught)] | length | tostring | . + "      " | .[0:6]) \([$x[] | .reason // empty] | unique | join(", "))" ),
    "",
    "hook disables/bypasses: \($hooks | length)\(if ($hooks | length) > 0 then " (" + ([$hooks[] | "\(.hook) \(.event) \(.detail)"] | join("; ")) + ")" else "" end)",
    "unreasoned or missing gates: \(if ($ucount | length) == 0 then "none" else ([$ucount[] | "\(.gate) ×\(.n)"] | join(", ")) end)",
    "host-reported tokens: \($tok // "not reported")",
    "",
    ( if ($done | length) < $n then "verdict: pending (\($done | length) of \($n) features finished)"
      else ( [ (if $catches < $minc then "catches \($catches) < \($minc) per \($n) features" else empty end),
               (if ($hooks | length) > $maxh then "hook disables \($hooks | length) > \($maxh)" else empty end),
               ($ucount[] | select(.n > $maxs) | "gate \(.gate) skipped without reason \(.n) > \($maxs) times") ] ) as $fail
           | if ($fail | length) == 0 then "verdict: KEEP" else "verdict: REVISIT (" + ($fail | join("; ")) + ")" end
      end )
' "$log"
