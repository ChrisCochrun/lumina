{
  description = "A Church Presentation Application";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
          # overlays = [cargo2nix.overlays.default];
        };
        inherit (pkgs) lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain fenix.packages.${system}.stable.toolchain;
        naersk' = pkgs.callPackage naersk { };

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
          cargo-nextest
          rust-analyzer
          vulkan-loader
          wayland
          wayland-protocols
          pkg-config
          sccache
        ];

        bi = with pkgs; [
          gcc
          stdenv
          gnumake
          gdb
          lldb
          cmake
          clang
          libclang
          libxkbcommon
          makeWrapper
          vulkan-headers
          vulkan-loader
          vulkan-tools
          libGL

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

        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
        ];

        ldLibPaths = "$LD_LIBRARY_PATH:${
          with pkgs;
          lib.makeLibraryPath [
            alsa-lib
            gst_all_1.gst-libav
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-bad
            gst_all_1.gst-plugins-good
            gst_all_1.gst-plugins-ugly
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-rs
            gst_all_1.gst-vaapi
            glib
            fontconfig
            vulkan-loader
            wayland
            wayland-protocols
            libxkbcommon
            mupdf
            libclang
          ]
        }";
 
        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = nbi;
          buildInputs = bi;
          LD_LIBRARY_PATH = ldLibPaths;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        lumina = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
              pkgs.sqlx-cli
            ];

            preBuild = ''
              export DATABASE_URL=sqlite:./db.sqlite3
              sqlx database create
              sqlx migrate run
            '';
          }
        );

      in
      rec {
        checks = {
          inherit lumina;
        };
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          inputsFrom = [ lumina ];
          packages = with pkgs; [
            sqlx-cli
            cargo-flamegraph
            bacon
            just
            cargo-watch
          ];
        };
        packages.default = lumina;
      }
    );
}
