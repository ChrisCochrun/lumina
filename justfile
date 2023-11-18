default:
    just --list
build:
    cmake -DCMAKE_BUILD_TYPE=Debug -B bld/ .
    make -j8 --dir bld/
    rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/
run:
    RUST_LOG=debug ./bld/bin/lumina
lint:
    cargo clippy
