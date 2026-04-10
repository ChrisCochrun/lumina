{
  description = "A Church Presentation Application";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          # overlays = [ rust-overlay.overlays.default ];
          # overlays = [cargo2nix.overlays.default];
        };
        inherit (pkgs) lib;
        craneLib = crane.mkLib pkgs;
        naersk' = pkgs.callPackage naersk { };

        # toolchain = (with pkgs.fenix.default; [cargo clippy rust-std rust-src rustc rustfmt rust-analyzer-nightly]);
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Include all the .sql migrations as well
            ./migrations
          ];
        };

        nativeBuildInputs = with pkgs; [
          # Rust tools
          # toolchain
          # (pkgs.fenix.default.withComponents [
          #   "cargo"
          #   "clippy"
          #   "rust-std"
          #   # "rust-src"
          #   "rustc"
          #   "rustfmt"
          # ])
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          }))
          cargo-nextest
          cargo-criterion
          # rust-analyzer-nightly
          vulkan-loader
          wayland
          wayland-protocols
          libxkbcommon
          pkg-config
          sccache
          just
          sqlx-cli
          cargo-watch
          samply
          flatpak-builder
        ];

        buildInputs = with pkgs; [
          gcc
          stdenv
          gnumake
          gdb
          lldb
          cmake
          clang
          libclang
          makeWrapper
          vulkan-headers
          vulkan-loader
          vulkan-tools
          libGL
          cargo-flamegraph
          bacon
          openssl

          fontconfig
          glib
          alsa-lib
          gst_all_1.gst-libav
          gst_all_1.gst-plugins-bad
          gst_all_1.gst-plugins-good
          gst_all_1.gst-plugins-ugly
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-rs
          gst_all_1.gst-vaapi
          gst_all_1.gstreamer
          ffmpeg-full
          mupdf
          # yt-dlp
        ];

        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
          with pkgs;
          pkgs.lib.makeLibraryPath [
            pkgs.alsa-lib
            pkgs.gst_all_1.gst-libav
            pkgs.gst_all_1.gstreamer
            pkgs.gst_all_1.gst-plugins-bad
            pkgs.gst_all_1.gst-plugins-good
            pkgs.gst_all_1.gst-plugins-ugly
            pkgs.gst_all_1.gst-plugins-base
            pkgs.gst_all_1.gst-plugins-rs
            pkgs.gst_all_1.gst-vaapi
            pkgs.glib
            pkgs.fontconfig
            pkgs.vulkan-loader
            pkgs.wayland
            pkgs.wayland-protocols
            pkgs.libxkbcommon
            pkgs.mupdf
            pkgs.libclang
          ]
        }";

        commonArgs = {
          strictDeps = false;
          inherit src buildInputs nativeBuildInputs LD_LIBRARY_PATH;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        lumina = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts buildInputs nativeBuildInputs LD_LIBRARY_PATH;

            preBuild = ''
              export DATABASE_URL=sqlite:./db.sqlite3
              sqlx database create
              sqlx migrate run
            '';
            cargoTestCommand = "";
            cargoExtraArgs = "";
          }
        );

      in
      rec {
        devShell =
          pkgs.mkShell.override
            {
              # stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
            }
            {
              inherit nativeBuildInputs buildInputs LD_LIBRARY_PATH;
              # LIBCLANG_PATH = "${pkgs.clang}";
              DATABASE_URL = "sqlite://./test.db";
              # RUST_SRC_PATH = "${toolchain.rust-src}/lib/rustlib/src/rust/library";
            };
        defaultPackage = lumina;
        packages = {
          default = lumina;
        };
      }
    );
}
