# Use powershell on Windows to avoid Git Bash's `link.exe` shadowing MSVC's linker

set windows-shell := ["pwsh", "-c"]

# Extract the workspace version from Cargo.toml
version := `grep -m1 '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/'`

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

# runs Swift runtime tests (macOS and Linux only)
[unix]
swift-test:
    @echo '{{ style("command") }}swift-test:{{ NORMAL }}'
    swift test --package-path crates/facet_generate/runtime/swift

[windows]
swift-test:
    @echo '{{ style("command") }}swift-test: skipped on Windows{{ NORMAL }}'

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

check-nursery:
    @echo '{{ style("command") }}check-nursery:{{ NORMAL }}'
    cargo clippy --all-targets -- --no-deps '-Dclippy::nursery' -Dwarnings

# local development: fix, check, build, test with snapshot review
dev: fix check build test-review

# builds documentation and fails on warnings
docs:
    @echo '{{ style("command") }}docs:{{ NORMAL }}'
    cargo rustdoc --all-features -p facet-generate-attrs -- -D warnings
    cargo rustdoc --all-features -p facet_generate -- -D warnings

# CI pipeline: check, build, test (matches .github/workflows/build.yaml)
ci: check docs build test swift-test

update-rust-deps:
    @echo '{{ style("command") }}update-rust-deps:{{ NORMAL }}'
    cargo update
    cargo upgrade --incompatible allow
    cargo update

# publish both crates to crates.io in dependency order, then tag and push
# Note: run `cargo login` first if you haven't already
publish:
    @echo '{{ style("command") }}publish v{{ version }}:{{ NORMAL }}'
    cargo publish -p facet-generate-attrs
    cargo publish -p facet_generate
    git tag -a "v{{ version }}" -m "Release v{{ version }}"
    git push origin "v{{ version }}"
