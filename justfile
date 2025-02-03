ui := "-i"
file := "~/dev/lumina-iced/test_presentation.lisp"

default:
    just --list
build:
    RUST_LOG=debug cargo build
run:
    RUST_LOG=debug cargo run -- {{ui}} {{file}}
clean:
    RUST_LOG=debug cargo clean
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture

alias r := run
alias c := clean
