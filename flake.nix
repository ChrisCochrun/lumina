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
            gtk-layer-shell
            gtk3
            vulkan-loader
            wayland
            wayland-protocols
            libxkbcommon
            pkg-config
          ];

          bi = [
            gcc
            stdenv
            gnumake
            gdb
            makeWrapper
            vulkan-headers
            vulkan-loader
            vulkan-tools
            libGL

            # podofo
            mpv
            ffmpeg-full
            # yt-dlp

            just
          ];
        in rec
        {
          devShell = pkgs.mkShell {
            nativeBuildInputs = nbi;
            buildInputs = bi;
          };
          defaultPackage = naersk'.buildPackage {
            src = ./.;
          };
        }
      );
}
