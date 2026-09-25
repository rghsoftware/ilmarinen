#!/bin/sh
# Tests for templates/scripts/trace.sh: dangling references fail, unverified
# Requirements warn, specscore:// resolves through repos.toml. Run under dash.
set -u
pkg=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
check() { if [ "$2" = "$3" ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $1 (want $2, got $3)"; fi; }
feature() { # dir slug
  mkdir -p "$1/spec/features/$2"
  printf '# Feature: %s\n\n#### REQ: a\n\n#### REQ: b\n\n### AC: a-ok (verifies REQ:a)\n' "$2" > "$1/spec/features/$2/README.md"
}
mkrepo() { # dir org/repo
  mkdir -p "$1/scripts"; git init -q "$1"; cp "$pkg/templates/scripts/trace.sh" "$1/scripts/"
  printf 'project:\n  host: github.com\n  org: %s\n  repo: %s\n' "${2%/*}" "${2#*/}" > "$1/specscore.yaml"
}
mkrepo "$T/other" acme/other; feature "$T/other" billing
mkrepo "$T/a" acme/a; feature "$T/a" login
export ILMARINEN_CONFIG=$T/cfg; mkdir -p "$T/cfg"
run() { (cd "$T/a" && sh scripts/trace.sh >"$T/out" 2>&1); echo $?; }

printf '// specscore:implements feature/login#req:a\n// specscore:verifies feature/login#ac:a-ok\n' > "$T/a/x.rs"
check "all resolve" 0 "$(run)"
grep -q 'feature/login#req:b has no specscore:verifies' "$T/out"; check "unverified REQ b warns" 0 $?
grep -q 'feature/login#req:a has no' "$T/out"; check "REQ a verified via AC" 1 $?
[ -f "$T/a/artifacts/traceability.md" ]; check "report written" 0 $?

printf '// specscore:implements feature/login#req:zzz\n' > "$T/a/y.rs"
check "dangling REQ fails" 1 "$(run)"
printf '# specscore:implements feature/nope#req:a\n' > "$T/a/y.rs"
check "dangling feature fails" 1 "$(run)"
printf '// specscore:references https://specscore.org/github.com/acme/a/spec/features/login#req:a\n' > "$T/a/y.rs"
check "expanded self URL resolves" 0 "$(run)"

printf '// specscore:references specscore://github.com/acme/other/feature/billing#req:b\n' > "$T/a/y.rs"
check "cross-repo without repos.toml fails" 1 "$(run)"
printf '[[repo]]\nid = "github.com/acme/other"\npath = "%s"\n' "$T/other" > "$T/cfg/repos.toml"
check "cross-repo via repos.toml resolves" 0 "$(run)"
printf '// specscore:references specscore://github.com/acme/other/feature/billing#ac:missing\n' > "$T/a/y.rs"
check "cross-repo dangling AC fails" 1 "$(run)"
printf '// specscore:frobnicates feature/login#req:a\n' > "$T/a/y.rs"
check "unknown verb fails" 1 "$(run)"
grep -q 'unknown verb frobnicates' "$T/out"; check "unknown verb named" 0 $?

echo "trace: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
