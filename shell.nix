{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell rec {
  name = "lumina";

  nativeBuildInputs = [
    gtk-layer-shell
    gtk3
    vulkan-loader
    wayland
    wayland-protocols
    libxkbcommon
    pkg-config
  ];

  buildInputs = [
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
    ffmpeg_5-full
    # yt-dlp

    # Rust tools
    clippy
    rustc
    cargo
    rustfmt
    rust-analyzer
  ];

  # cargoDeps = rustPlatform.importCargoLock {
  #   lockFile = ./Cargo.lock;
  # };

  RUST_BACKTRACE = "full";
  CMAKE_C_COMPILER = "${gcc}/bin/gcc";
  CMAKE_CXX_COMPILER = "${gcc}/bin/g++";
  # This creates the proper qt env so that plugins are found right.
  # shellHook = ''
  #   setQtEnvironment=$(mktemp --suffix .setQtEnvironment.sh)
  #   echo "shellHook: setQtEnvironment = $setQtEnvironment"
  #   makeQtWrapper "/bin/sh" "$setQtEnvironment" "''${qtWrapperArgs[@]}"
  #   sed "/^exec/d" -i "$setQtEnvironment"
  #   source "$setQtEnvironment"
  #   export QT_PLUGIN_PATH="$QT_PLUGIN_PATH:/nix/store/85jx8w2nh1ln4kb0hf3dc6ky0dh6ri24-lightly-qt-0.4.1/lib/qt-5.15.9/plugins"
  # '';
}
