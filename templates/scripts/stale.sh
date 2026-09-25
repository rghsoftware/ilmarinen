#!/bin/sh
# Lessons still `Recorded` six weeks after their **Date:** with no
# **Promotes To:** (docs/ILMARINEN.md, Compounding). Lists them; exits 0.
set -eu
cd "$(git rev-parse --show-toplevel)"
cutoff=$(date -u -d '42 days ago' +%F 2>/dev/null || date -u -v-42d +%F)
n=0
for f in spec/lessons/*/README.md; do
  [ -f "$f" ] || continue
  status=$(sed -n 's/^\*\*Status:\*\* *//p' "$f" | head -n 1)
  day=$(sed -n 's/^\*\*Date:\*\* *//p' "$f" | head -n 1)
  promoted=$(sed -n 's/^\*\*Promotes To:\*\* *//p' "$f" | head -n 1)
  d=$(printf '%s' "$day" | tr -d -); c=$(printf '%s' "$cutoff" | tr -d -)
  case "$d" in ''|*[!0-9]*) continue ;; esac
  if [ "$status" = Recorded ] && { [ -z "$promoted" ] || [ "$promoted" = "—" ]; } && [ "$d" -lt "$c" ]; then
    slug=${f#spec/lessons/}; echo "stale: ${slug%/README.md} (Recorded $day, not promoted)"; n=$((n + 1))
  fi
done
echo "stale: $n Lesson(s) older than $cutoff without promotion; propose deleting them at review"
