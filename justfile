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
    cargo run -- {{verbose}}
run-release:
    cargo run --release -- {{verbose}}
run-file:
    cargo run -- {{verbose}} cli {{file}}
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
    cargo build --release
    cargo packager --release -f nsis

# export DYLD_FALLBACK_LIBRARY_PATH="/Library/Frameworks/GStreamer.framework/Libraries"
mac-packager:
    cargo install cargo-packager --locked
    export PKG_CONFIG_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib/pkgconfig
    export PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/bin:$PATH
    cargo build --release
    install_name_tool -add_rpath @executable_path/../Frameworks target/release/lumina
    cargo packager --release -f dmg
    # install_name_tool -change libgio-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgio-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgstpbutils-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgstpbutils-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgstvideo-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgstvideo-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgstaudio-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgstaudio-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgstbase-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgstbase-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgstreamer-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgstreamer-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgobject-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgobject-2.0.0.dylib target/release/lumina
    # install_name_tool -change libglib-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libglib-2.0.0.dylib target/release/lumina
    # install_name_tool -change libgstapp-2.0.0.dylib @loader_path/../Frameworks/GStreamer.framework/Libraries/libgstapp-2.0.0.dylib target/release/lumina

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
    flatpak-builder --install-deps-from=flathub --keep-build-dirs --install --user --force-clean build-dir xyz.cochrun.lumina.yml

flatpak-shell:
    flatpak-builder --run build-dir xyz.cochrun.lumina.yml sh

alias fb := flatpak-build
alias fs := flatpak-setup
