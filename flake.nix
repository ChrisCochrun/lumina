{
  description = "A Church Presentation Application";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
            # overlays = [cargo2nix.overlays.default];
          };
          naersk' = pkgs.callPackage naersk {};
          nbi = with pkgs; [
            # Rust tools
            alejandra
            (pkgs.fenix.stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            rust-analyzer
            vulkan-loader
            wayland
            wayland-protocols
            libxkbcommon
            pkg-config
            sccache
          ];

          bi = with pkgs; [
            gcc
            stdenv
            gnumake
            gdb
            cmake
            makeWrapper
            vulkan-headers
            vulkan-loader
            vulkan-tools
            libGL
            cargo-flamegraph

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
            # podofo
            # mpv
            ffmpeg-full
            # yt-dlp

            just
            sqlx-cli
            cargo-watch
          ];
        in rec
        {
          devShell = pkgs.mkShell.override {
            # stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
          } {
            nativeBuildInputs = nbi;
            buildInputs = bi;
            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
              with pkgs;
              pkgs.lib.makeLibraryPath [
                pkgs.vulkan-loader
                pkgs.wayland
                pkgs.wayland-protocols
                pkgs.libxkbcommon
              ]
            }";
            DATABASE_URL = "sqlite:///home/chris/.local/share/lumina/library-db.sqlite3";
          };
          defaultPackage = naersk'.buildPackage {
            src = ./.;
          };
          packages = {
            default = naersk'.buildPackage {
              src = ./.;
            };
          };
        }
      );
}
