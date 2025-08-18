ui := "-i"
file := "~/dev/lumina-iced/test_presentation.lisp"

default:
    just --list
build:
    RUST_LOG=debug cargo build
sbuild:
    RUST_LOG=debug sccache cargo build
run:
    RUST_LOG=debug cargo run -- {{ui}} {{file}}
srun:
    RUST_LOG=debug sccache cargo run -- {{ui}} {{file}}
clean:
    RUST_LOG=debug cargo clean
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture
profile:
    cargo flamegraph --image-width 8000 -- {{ui}} {{file}}

alias b := build
alias r := run
alias sr := srun
alias c := clean
