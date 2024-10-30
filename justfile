default:
    just --list
build:
    RUST_LOG=debug cargo build
run: 
    RUST_LOG=debug cargo run -- ~/dev/lumina-iced/test_presentation.lisp
clean:
    RUST_LOG=debug cargo clean
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture
