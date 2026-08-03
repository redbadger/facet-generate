# Use powershell on Windows to avoid Git Bash's `link.exe` shadowing MSVC's linker

set windows-shell := ["pwsh", "-c"]

# Extract the workspace version from Cargo.toml
version := `grep -m1 '^version' crates/facet_generate/Cargo.toml | sed 's/version = "\(.*\)"/\1/'`
attrs-version := `grep -m1 '^version' crates/facet-generate-attrs/Cargo.toml | sed 's/version = "\(.*\)"/\1/'`

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

# Release flow:
#
#   1. bump the version in the crate's Cargo.toml, and date the CHANGELOG
#      section (`## [x.y.z] - unreleased` -> `## [x.y.z] - <date>`)
#   2. `just ci`, then commit the bump and push it to `main`
#   3. create the GitHub release, which creates the tag for you:
#        gh release create "facet-generate-v<version>" --target <commit> \
#          --notes-file <the CHANGELOG section>
#      or, when releasing without a GitHub release, `just tag`
#   4. `just publish`
#
# Tagging is deliberately not part of `publish`: `gh release create` already
# creates the tag, so doing it in `publish` too fails the whole recipe after
# the irreversible upload has already happened. Keeping them separate means
# `publish` behaves the same whichever order you release in.

# publish the main crate to crates.io
# Note: run `cargo login` first if you haven't already
publish:
    @echo '{{ style("command") }}publish v{{ version }}:{{ NORMAL }}'
    cargo publish -p facet_generate

# publish the attribute macro crate independently (only needed when it changes)
# Note: run `cargo login` first if you haven't already
publish-attrs:
    @echo '{{ style("command") }}publish facet-generate-attrs v{{ attrs-version }}:{{ NORMAL }}'
    cargo publish -p facet-generate-attrs

# tag git HEAD and push the tag (unnecessary if a GitHub release made it; HEAD is `@-` under jj)
tag:
    @echo '{{ style("command") }}tag facet-generate-v{{ version }}:{{ NORMAL }}'
    git tag -a "facet-generate-v{{ version }}" -m "Release facet-generate v{{ version }}"
    git push origin "facet-generate-v{{ version }}"

# as `tag`, for the independently versioned attribute macro crate
tag-attrs:
    @echo '{{ style("command") }}tag facet-generate-attrs-v{{ attrs-version }}:{{ NORMAL }}'
    git tag -a "facet-generate-attrs-v{{ attrs-version }}" -m "Release facet-generate-attrs v{{ attrs-version }}"
    git push origin "facet-generate-attrs-v{{ attrs-version }}"
