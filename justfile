ui := "-i"
verbose := "-v"
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
    cargo run -- {{verbose}} {{ui}}
run-release:
    cargo run --release -- {{verbose}} {{ui}}
run-file:
    cargo run -- {{verbose}} {{ui}} {{file}}
clean:
    cargo clean
test:
    cargo nextest run
ci-test:
    cargo nextest run -- --skip test_db_and_model --skip test_update --skip test_song_slide_speed --skip test_song_to_slide --skip test_song_from_db
bench:
    export NEXTEST_EXPERIMENTAL_BENCHMARKS=1
    cargo nextest bench
profile:
    samply record cargo run --release -- {{verbose}} {{ui}}

alias b := build
alias r := run
alias br := build-release
alias rr := run-release
alias rf := run-file
alias c := clean
