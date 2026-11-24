[windows]
set shell := ["nu", "-c"]

verbose := "-v"
file := "~/dev/lumina-iced/test_presentation.lisp"
sdk-version := "25.08"

# export RUSTC_WRAPPER := "sccache"
# export RUST_LOG := "debug"

default:
    just --list
build:
    cargo build -p lumina
build-release:
    cargo build -p lumina --release
build-offline:
    cargo build -p lumina --release --offline
run:
    cargo run -p lumina -- {{verbose}}
run-release:
    cargo run -p lumina --release -- {{verbose}}
run-file:
    cargo run -p lumina -- {{verbose}} cli {{file}}
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
    samply record cargo run --release -- {{verbose}}

alias b := build
alias r := run
alias br := build-release
alias rr := run-release
alias rf := run-file
alias c := clean

##### Sets up and builds the exe installer with nsis
windows-packager:
    cargo install cargo-packager --locked
    cargo xtask gstreamer stage --expected-version 1.28.8
    cargo xtask package windows --expected-version 1.28.8 --msvc-toolset v145
    # cargo packager --release -f nsis

##### Sets up and builds the macos bundle and dmg
gst := "/Library/Frameworks/GStreamer.framework/Versions/1.0"
[env("PKG_CONFIG", gst + "/bin/pkg-config")]
[env("PKG_CONFIG_PATH", gst + "/lib/pkgconfig")]
[env("PKG_CONFIG_LIBDIR", gst + "/lib/pkgconfig")]
[env("PKG_CONFIG_ALL_DYNAMIC", "1")]
[env("PATH", gst + "/bin:" + env("PATH"))]
[env("BIN", "target/release/lumina")]
[env("RPATH", "@executable_path/../Frameworks/GStreamer.framework/Libraries")]
mac-packager:
    cargo install cargo-packager --locked
    # Verify GStreamer configuration.
    echo "Using pkg-config:"
    command -v pkg-config

    echo "GStreamer pkg-config location:"
    pkg-config --variable=pcfiledir gstreamer-1.0

    echo "GStreamer library directory:"
    pkg-config --variable=libdir gstreamer-1.0

    echo "GStreamer linker flags:"
    pkg-config --libs gstreamer-1.0

    # Build Lumina.
    cargo build -p lumina --release
    
    # Check for accidental MacPorts dependencies.
    if otool -L "$BIN" | grep -F "/opt/local/lib/"; then
    echo "ERROR: Lumina still references MacPorts libraries."
    exit 1
    fi
    
    # Add the bundled framework's library search path.
    if ! otool -l "$BIN" | grep -Fq "path $RPATH ("; then
    install_name_tool -add_rpath "$RPATH" "$BIN"
    fi
    # Create the DMG using the existing Cargo.toml config.
    cargo packager --release -f dmg

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
    python3 split_flatpak_cargo_sources.py cargo-sources.json cargo-sources-split.json

flatpak-build:
    flatpak-builder --install-deps-from=flathub --keep-build-dirs --install --user --force-clean build-dir xyz.cochrun.lumina.yml

flatpak-shell:
    flatpak-builder --run build-dir xyz.cochrun.lumina.yml sh

flatpak-create:
    flatpak-builder --user --force-clean --install-deps-from=flathub --repo=repo build-dir xyz.cochrun.lumina.yml
    flatpak build-bundle --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo repo lumina.flatpak xyz.cochrun.lumina
    
alias fb := flatpak-build
alias fs := flatpak-setup
