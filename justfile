ui := "-i"
file := "~/dev/lumina-iced/test_presentation.lisp"

export RUSTC_WRAPPER := "sccache"
# export RUST_LOG := "debug"

default:
    just --list
build:
    cargo build
build-release:
    cargo build --release
run:
    cargo run -- {{ui}}
run-release:
    cargo run --release -- {{ui}}
run-file:
    cargo run -- {{ui}} {{file}}
clean:
    cargo clean
test:
    cargo test --benches --tests --all-features -- --nocapture
profile:
    cargo flamegraph -- {{ui}}

alias b := build
alias r := run
alias br := build-release
alias rr := run-release
alias rf := run-file
alias c := clean
