{
  description = "A Church Presentation Application";

  inputs = {
    # cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    # nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            # overlays = [cargo2nix.overlays.default];
          };
          naersk' = pkgs.callPackage naersk {};
          # src = ./.;
          # rustPkgs = pkgs.rustBuilder.makePackageSet {
          #   rustVersion = "1.61.0";
          #   packageFun = import ./Cargo.nix;
          # };
          # The workspace defines a development shell with all of the dependencies
          # and environment settings necessary for a regular `cargo build`.
          # Passes through all arguments to pkgs.mkShell for adding supplemental
          # dependencies.
          # workspaceShell = rustPkgs.workspaceShell {
          #   packages = with pkgs; [
          #     gcc
          #     stdenv
          #     bintools
          #     gnumake
          #     gdb
          #     qtcreator
          #     cmake
          #     extra-cmake-modules
          #     pkg-config
          #     libsForQt5.wrapQtAppsHook
          #     makeWrapper

          #     clang-tools
          #     clang
          #     libclang
          #     qt5.qtbase
          #     qt5.qttools
          #     qt5.qtquickcontrols2
          #     qt5.qtx11extras
          #     qt5.qtmultimedia
          #     qt5.qtwayland
          #     qt5.qtwebengine
          #     libsForQt5.kirigami2
          #     libsForQt5.qqc2-desktop-style
          #     libsForQt5.karchive
          #     mpv
          #     ffmpeg_6-full
          #     # Rust tools
          #     clippy
          #     rustc
          #     cargo
          #     rustfmt
          #     rust-analyzer
          #     corrosion
          #   ];
          #   # shellHook = ''
          #   #   export PS1="\033[0;31m☠dev-shell☠ $ \033[0m"
          #   # '';
          # };

        in rec
        {
          # packages = {
          #   crate = (rustPkgs.workspace.libre-presenter {  }).bin;
          #   default = packages.crate;
          # };
          devShell = import ./shell.nix { inherit pkgs; };
          defaultPackage = pkgs.libsForQt5.callPackage ./default.nix { };
        }
      );
}
