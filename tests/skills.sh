#!/bin/sh
# Consistency checks across skill texts. Run under dash.
set -u
S=$(cd "$(dirname "$0")/../plugin/skills" && pwd)
pass=0; fail=0
check() { if grep -q -- "$2" "$S/$1/SKILL.md"; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $1 lacks: $2"; fi; }
# Lesson close-test-issues-when-tests-land: plan orders tests first, so a
# tests-first issue must close when its tests land, or the loop deadlocks.
check plan "closed, when its failing tests are committed"
check forge 'bd close <id> --reason "failing tests in <commit>"'
check verify "closes the implementing issue"
for s in route blueprint rune plan forge verify sampo survey; do
  n=$(sed -n 's/^name: //p' "$S/$s/SKILL.md"); [ "$n" = "$s" ] && pass=$((pass + 1)) || { fail=$((fail + 1)); echo "FAIL: $s name '$n'"; }
done
echo "skills: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
