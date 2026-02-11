{
  description = "A Church Presentation Application";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    rust-overlay.url = "github:oxalica/rust-overlay";
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
        naersk' = pkgs.callPackage naersk { };

        # toolchain = (with pkgs.fenix.default; [cargo clippy rust-std rust-src rustc rustfmt rust-analyzer-nightly]);
            

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

          just
          sqlx-cli
          cargo-watch
          samply
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
        defaultPackage = naersk'.buildPackage {
          inherit nativeBuildInputs buildInputs LD_LIBRARY_PATH;
          src = ./.;
        };
        packages = {
          default = naersk'.buildPackage {
            inherit nativeBuildInputs buildInputs LD_LIBRARY_PATH;
            src = ./.;
          };
        };
      }
    );
}
