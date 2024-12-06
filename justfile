default:
    just --list
build:
    RUST_LOG=debug cargo build
run ui=' ' file='~/dev/lumina-iced/test_presentation.lisp': 
    RUST_LOG=debug cargo run -- -i {{ui}} {{file}}
clean:
    RUST_LOG=debug cargo clean
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture

alias r := run
alias c := clean
