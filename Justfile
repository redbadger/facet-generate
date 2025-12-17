build:
    cargo build --all-features

test:
    cargo insta test --review --test-runner nextest --all-features

lint:
    cargo clippy --all-targets -- --no-deps -Dclippy::pedantic -Dwarnings
