#!/bin/sh
# Tests for templates/scripts/scorecard.sh and retro.sh. Run under dash.
set -u
pkg=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
check() { if [ "$2" = "$3" ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $1 (want $2, got $3)"; fi; }
has() { if grep -q -- "$2" "$T/out"; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $1 (no '$2')"; cat "$T/out"; fi; }
git init -q "$T/r"; mkdir -p "$T/r/scripts"; cp "$pkg/templates/scripts/scorecard.sh" "$pkg/templates/scripts/retro.sh" "$T/r/scripts/"
cp "$pkg/templates/scorecard.toml" "$T/r/"; cd "$T/r"
sc() { sh scripts/scorecard.sh "$@" 2>/dev/null; echo $?; }
retro() { sh scripts/retro.sh > "$T/out" 2>&1; }

check "skip without reason refused" 2 "$(sc gate plan f1 skipped)"
check "multi-word reason refused" 2 "$(sc gate plan f1 skipped --reason "too small")"
check "unknown gate refused" 2 "$(sc gate lint f1 fired)"
check "bad depth refused" 2 "$(sc done f1 huge)"
check "caught skip refused" 2 "$(sc gate plan f1 skipped --reason x --caught)"
[ ! -s artifacts/scorecard.jsonl ]; check "refusals write nothing" 0 $?
retro; has "empty log" "no events yet"

full() { # feature depth [caught-gate]
  for g in checks review merge; do sc gate $g "$1" fired >/dev/null; done
  [ "$2" = trivial ] || sc gate plan "$1" fired >/dev/null
  [ "$2" = major ] && { sc gate intent "$1" fired ${3:+--caught} >/dev/null; sc gate decision "$1" skipped --reason no-choice >/dev/null; }
  sc done "$1" "$2" >/dev/null
}
full f1 major yes
jq -e 'select(.gate=="intent") | .caught == true and .v == 1' artifacts/scorecard.jsonl >/dev/null; check "event schema" 0 $?
retro; has "pending verdict" "verdict: pending (1 of 5"
has "decision skip reason shown" "no-choice"
for f in f2 f3 f4 f5; do full $f standard; done
retro; has "keep with one catch" "verdict: KEEP"

: > artifacts/scorecard.jsonl
for f in a b c d e; do full $f trivial; done
retro; has "revisit on no catches" "catches 0 < 1 per 5 features"
sc hook leak-guard bypassed no-verify >/dev/null; sc hook leak-guard bypassed no-verify >/dev/null; sc hook pre-commit disabled guard-missing >/dev/null
retro; has "revisit on hook disables" "hook disables 3 > 2"

: > artifacts/scorecard.jsonl
for f in a b c; do sc gate checks $f fired --caught >/dev/null; sc gate review $f fired >/dev/null; sc done $f standard >/dev/null; done
for f in d e; do full $f standard; done
retro; has "missing gates counted" "gate plan skipped without reason 3 > 2"
has "missing merge counted" "gate merge skipped without reason 3 > 2"

: > artifacts/scorecard.jsonl
full f1 major yes; sc done f1 major >/dev/null
retro; has "a feature finished twice counts once" "verdict: pending (1 of 5"

echo "scorecard: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
