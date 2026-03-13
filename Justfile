# Use powershell on Windows to avoid Git Bash's `link.exe` shadowing MSVC's linker

set windows-shell := ["pwsh", "-c"]

# default target for local development
default: dev

# builds all crates
build:
    @echo '{{ style("command") }}build:{{ NORMAL }}'
    cargo build --all-features

clean:
    @echo '{{ style("command") }}clean:{{ NORMAL }}'
    cargo clean
    rm -rf crates/facet_generate/runtime/swift/.build

# runs tests
test:
    @echo '{{ style("command") }}test:{{ NORMAL }}'
    cargo nextest run --all-features

# runs tests with snapshot review (interactive, for local dev)
test-review:
    @echo '{{ style("command") }}test-review:{{ NORMAL }}'
    cargo insta test --review --test-runner nextest --all-features

# auto-fix formatting issues
fix:
    @echo '{{ style("command") }}fix:{{ NORMAL }}'
    cargo fmt --all

# validate formatting and lint (strict, no auto-fix)
check:
    @echo '{{ style("command") }}check:{{ NORMAL }}'
    cargo fmt --all -- --check
    cargo clippy --all-targets -- --no-deps '-Dclippy::pedantic' -Dwarnings

# local development: fix, check, build, test with snapshot review
dev: fix check build test-review

# CI pipeline: check, build, test (matches .github/workflows/build.yaml)
ci: check build test

update-rust-deps:
    @echo '{{ style("command") }}update-rust-deps:{{ NORMAL }}'
    cargo update
    cargo upgrade --incompatible allow
    cargo update
