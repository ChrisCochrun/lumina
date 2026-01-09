ui := "-i"
file := "~/dev/lumina-iced/test_presentation.lisp"

default:
    just --list
build:
    RUST_LOG=debug cargo build
sbuild:
    RUST_LOG=debug sccache cargo build
run:
    RUST_LOG=debug cargo run -- {{ui}}
run-file:
    RUST_LOG=debug cargo run -- {{ui}} {{file}}
srun:
    RUST_LOG=debug sccache cargo run -- {{ui}} {{file}}
clean:
    RUST_LOG=debug cargo clean
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture
profile:
    cargo flamegraph -- {{ui}}

alias b := build
alias r := run
alias rf := run-file
alias sr := srun
alias c := clean
