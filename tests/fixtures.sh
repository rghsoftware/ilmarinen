#!/bin/sh
# For each fixture: copy to a temp git repo, `ilmarinen init`, install the
# project's own dependencies, then `just check`, `specscore spec lint` and
# `scripts/trace.sh`. With FIXTURES_FULL=1 also `just test`, `just e2e` and
# `just db-up` / `just db-down` (needs docker). Usage: tests/fixtures.sh [name...].
# Tools come from PATH (mise); ILMARINEN_BIN overrides the CLI binary.
set -eu
pkg=$(cd "$(dirname "$0")/.." && pwd)
bin=${ILMARINEN_BIN:-$pkg/cli/target/debug/ilmarinen}
[ -x "$bin" ] || { echo "fixtures: $bin not built (just build or cargo build)" >&2; exit 1; }
[ $# -gt 0 ] || set -- $(cd "$pkg/fixtures" && ls -d */ | tr -d /)
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
export ILMARINEN_CONFIG=$work/config SPECSCORE_TELEMETRY=0 BD_DISABLE_METRICS=1 DO_NOT_TRACK=1
mkdir -p "$ILMARINEN_CONFIG"; echo 'opt_in_keyword = "full-depth"' > "$ILMARINEN_CONFIG/config.toml"
# Regression (Lesson bd-init-before-stamping): bd init commits AGENTS.md,
# CLAUDE.md, .claude/settings.json and .gitignore by name, so init refuses
# while any of them has uncommitted changes.
g=$work/guard; mkdir -p "$g"; git init -q "$g"; git -C "$g" config user.email f@example.invalid; git -C "$g" config user.name f
echo "# mine" > "$g/AGENTS.md"; git -C "$g" add AGENTS.md; git -C "$g" commit -qm base; echo "edit" >> "$g/AGENTS.md"
if "$bin" init "$g" >"$work/guard.out" 2>&1 || ! grep -q "bd init commits these paths" "$work/guard.out" || [ -d "$g/.beads" ]; then
  echo "fixtures: init did not refuse a dirty AGENTS.md" >&2; cat "$work/guard.out" >&2; exit 1
fi
echo "fixtures: init refuses dirty bd-committed paths"
failed=
for f in "$@"; do
  echo "=== fixture $f"
  r=$work/$f; mkdir -p "$r"; cp -R "$pkg/fixtures/$f/." "$r/"
  (
    cd "$r"
    git init -q && git config user.email fixture@example.invalid && git config user.name fixture
    git remote add origin "https://github.com/ilmarinen-fixtures/$f.git"
    git add -A && git commit -qm fixture
    "$bin" init .
    # Beads' own init commit may carry only .beads/ and its .gitignore block.
    extra=$(git show --name-only --format= HEAD | grep -v -e '^\.beads/' -e '^\.gitignore$' || true)
    [ -z "$extra" ] || { echo "bd init committed non-Beads files: $extra" >&2; exit 1; }
    for d in $(find . -name package.json -not -path '*/node_modules/*' -exec dirname {} \;); do (cd "$d" && pnpm install --frozen-lockfile --silent); done
    for d in $(find . -name pyproject.toml -not -path '*/.venv/*' -exec dirname {} \;); do (cd "$d" && uv sync --frozen --quiet); done
    just check
    specscore spec lint
    scripts/trace.sh
    grep -q 'ILMARINEN LEAK GUARD' "$(git rev-parse --path-format=absolute --git-path hooks)/pre-commit"
    if [ "${FIXTURES_FULL:-0}" = 1 ]; then
      just test
      just e2e
      just db-up
      just db-down
    fi
    # Decision 0014: `ilmarinen off` makes the guard and every hook dormant;
    # `ilmarinen on` restores them. The address is assembled at runtime.
    leak=$(printf '%s.%s.%s.%s' 10 20 30 40)
    "$bin" off . && [ -f .ilmarinen.off ]
    printf 'x %s\n' "$leak" > ilm-dormant.txt; git add ilm-dormant.txt
    git commit -qm "off: guard dormant" || { echo "off: leak guard still blocked" >&2; exit 1; }
    o=$(printf '{"tool_name":"Bash","cwd":"%s","tool_input":{"command":"git push --force"}}' "$PWD" | sh "$pkg/plugin/hooks/pre-tool-deny.sh" 2>&1) \
      && [ -z "$o" ] || { echo "off: deny hook not silent: $o" >&2; exit 1; }
    "$bin" on . && [ ! -e .ilmarinen.off ]
    printf 'y %s\n' "$leak" >> ilm-dormant.txt; git add ilm-dormant.txt
    if git commit -qm "on: guard active" 2>/dev/null; then echo "on: leak guard did not block" >&2; exit 1; fi
    git reset -q HEAD~1 && rm -f ilm-dormant.txt
    echo "fixture $f: off/on verified"
  ) || failed="$failed $f"
done
[ -z "$failed" ] || { echo "fixtures FAILED:$failed" >&2; exit 1; }
echo "fixtures: all passed ($*)"
