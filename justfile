default:
    just --list
build:
    cmake -DCMAKE_BUILD_TYPE=Debug -B bld/ .
    make -j8 --dir bld/
    rm -rf ~/.cache/lumina/lumina/qmlcache/
run:
    RUST_LOG=debug ./bld/bin/lumina
lint:
    cargo clippy
clean:
    cargo clean
    rm -rf bld/
test:
    RUST_LOG=debug cargo test --benches --tests --all-features -- --nocapture