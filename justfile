ui := "-i"
verbose := "-v"
file := "~/dev/lumina-iced/test_presentation.lisp"
sdk-version := "25.08"

# export RUSTC_WRAPPER := "sccache"
# export RUST_LOG := "debug"

default:
    just --list
build:
    cargo build
build-release:
    cargo build --release
build-offline:
    cargo build --release --offline
run:
    cargo run -- {{verbose}} {{ui}}
run-release:
    cargo run --release -- {{verbose}} {{ui}}
run-file:
    cargo run -- {{verbose}} {{ui}} {{file}}
fix:
    cargo clippy --fix --bin "lumina" -p lumina -- -W clippy::pedantic -W clippy::perf -W clippy::nursery -W clippy::unwrap_used
clean:
    cargo clean
watch-clippy:
    cargo watch --why -x "clippy --all-targets --all-features"
test:
    cargo nextest run
ci-test:
    cargo nextest run -- --skip test_db_and_model --skip test_update --skip test_song_slide_speed --skip test_song_to_slide --skip test_song_from_db --skip song_search
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


##### Sets up flatpak to be able to build the lumina flatpak using all the latest pieces
flatpak-setup: flatpak-install-sdk install-flatpak-builder-tools
    git -C "cosmic-flatpak-runtime" pull || git clone https://github.com/pop-os/cosmic-flatpak-runtime.git "cosmic-flatpak-runtime"
    cd cosmic-flatpak-runtime
    flatpak-builder --install --user --force-clean build-dir cosmic-flatpak-runtime/com.system76.Cosmic.Sdk.json
    flatpak-builder --install --user --force-clean build-dir cosmic-flatpak-runtime/com.system76.Cosmic.BaseApp.json

flatpak-install-sdk:
    flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
    flatpak install --noninteractive --user flathub \
        org.freedesktop.Platform//{{ sdk-version }} \
        org.freedesktop.Platform.Locale//{{ sdk-version }} \
        org.freedesktop.Sdk//{{ sdk-version }} \
        org.freedesktop.Sdk.Locale//{{ sdk-version }} \
        org.freedesktop.Sdk.Docs//{{ sdk-version }} \
        org.freedesktop.Sdk.Debug//{{ sdk-version }} \
        org.freedesktop.Sdk.Extension.rust-nightly//{{ sdk-version }} \
        org.freedesktop.Sdk.Extension.llvm22//{{ sdk-version }}

install-flatpak-builder-tools:
    rm -rf flatpak-builder-tools
    git clone https://github.com/flatpak/flatpak-builder-tools --branch master --depth 1
    # pip install aiohttp tomlkit # Would be needed without nix

flatpak-gen-manifest: install-flatpak-builder-tools
    python3 flatpak-builder-tools/cargo/flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json

flatpak-build:
    flatpak-builder --install --user --force-clean build-dir xyz.cochrun.lumina.yml

alias fb := flatpak-build
alias fs := flatpak-setup
