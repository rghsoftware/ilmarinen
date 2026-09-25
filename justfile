# Gates for the Ilmarinen package itself. Hooks and CI call these targets.

set shell := ["bash", "-euo", "pipefail", "-c"]

# Per-run telemetry opt-outs (tools.toml [tool.*.telemetry] env).
export SPECSCORE_TELEMETRY := "0"
export BD_DISABLE_METRICS := "1"
export SEMGREP_SEND_METRICS := "off"
export MISE_USE_VERSIONS_HOST_TRACK := "0"
export DO_NOT_TRACK := "1"

target := "x86_64-unknown-linux-gnu"

default:
    @just --list

# Every deterministic check. `lang` scopes it (rust | spec | all); the
# post-edit hook passes the edited file's language.
check lang="all":
    @if [ "{{lang}}" = all ] || [ "{{lang}}" = rust ]; then cd cli && cargo fmt --check && cargo clippy --all-targets --features dev -- -D warnings; fi
    @if [ "{{lang}}" = all ] || [ "{{lang}}" = spec ]; then specscore spec lint; fi
    @if [ "{{lang}}" = all ]; then just convert-check >/dev/null; fi
    @if [ "{{lang}}" = all ] && command -v claude >/dev/null; then claude plugin validate . >/dev/null && claude plugin validate plugin >/dev/null && echo "plugin manifests valid"; elif [ "{{lang}}" = all ]; then echo "claude not installed (dev scope): plugin manifest validation skipped"; fi

# Unit tests, hook tests (POSIX sh), and the generated OpenCode plugin (bun is a
# dev-only dependency of this repo, not a tool Ilmarinen installs).
test:
    cd cli && cargo test --features dev
    dash tests/hooks.sh
    dash tests/trace.sh
    dash tests/scorecard.sh
    bun test tests/opencode-plugin.test.ts

# Regenerate hosts/opencode/ from plugin/ (an Artifact; never hand-edit it).
convert:
    cd cli && cargo run -q --features dev --bin ilmarinen-convert -- ../plugin ../hosts/opencode

# Fail if hosts/opencode/ is stale relative to plugin/.
convert-check:
    cd cli && cargo run -q --features dev --bin ilmarinen-convert -- --check ../plugin ../hosts/opencode

# Every fixture: init, dependencies, just check, spec lint, trace (slow; CI runs it).
fixtures *names:
    cd cli && cargo build -q
    tests/fixtures.sh {{names}}

# doctor in CI mode: ci/all scope tools only, no hosts, no MCP, no per-machine state.
# The binaries under test are the ones just built, so they go first on PATH.
doctor-ci:
    cd cli && cargo build -q --bin ilmarinen --bin ilmarinen-sampo && PATH="$PWD/target/debug:$PATH" target/debug/ilmarinen doctor --skip-mcp

# Static release binary (glibc crt-static; no extra rustup target needed).
build:
    cd cli && RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target {{target}}
    @echo "cli/target/{{target}}/release/ilmarinen"
