default:
    just --list
build:
    RUST_LOG=debug cargo build
run ui=' ' file='~/dev/lumina-iced/testypres.lisp': 
    RUST_LOG=debug cargo run -- {{ui}} {{file}}
clean:
    RUST_LOG=debug cargo clean
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture
