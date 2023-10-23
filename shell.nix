{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell rec {
  name = "lumina";

  nativeBuildInputs = [
    # ffmpeg
    gcc
  ];

  buildInputs = [
    stdenv
    gnumake
    gdb
    qtcreator
    cmake
    extra-cmake-modules
    pkg-config
    libsForQt5.wrapQtAppsHook
    makeWrapper


    clang-tools
    clang
    libclang
    # clang-format
    qt5.qtbase
    qt5.qttools
    qt5.qtquickcontrols2
    qt5.qtx11extras
    qt5.qtmultimedia
    qt5.qtwayland
    qt5.qtwebengine
    libsForQt5.kirigami2
    # libsForQt5.breeze-icons
    # libsForQt5.breeze-qt5
    libsForQt5.qqc2-desktop-style
    libsForQt5.karchive
    libsForQt5.sonnet
    # libsForQt5.kirigami-addons
    # libsForQt5.ki18n
    # libsForQt5.kcoreaddons
    # libsForQt5.kguiaddons
    # libsForQt5.kconfig

    # podofo
    mpv
    ffmpeg_5-full
    # yt-dlp

    # Rust tools
    just
    clippy
    rustc
    cargo
    rustfmt
    rust-analyzer
    corrosion
  ];

  # cargoDeps = rustPlatform.importCargoLock {
  #   lockFile = ./Cargo.lock;
  # };

  RUST_BACKTRACE = "full";
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  CMAKE_C_COMPILER = "${gcc}/bin/gcc";
  CMAKE_CXX_COMPILER = "${gcc}/bin/g++";

  CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG = true;
  # QT_SCALE_FACTOR = 1;
  # QT_PLUGIN_PATH="${QT_PLUGIN_PATH/':''/nix/store/85jx8w2nh1ln4kb0hf3dc6ky0dh6ri24-lightly-qt-0.4.1/lib/qt-5.15.9/plugins'':'/':'}"
    # QML2_IMPORT_PATH=${QML2_IMPORT_PATH/':''/run/current-system/sw/lib/qt-5.15.10/qml'':'/':'}

  # This creates the proper qt env so that plugins are found right.
  shellHook = ''
    setQtEnvironment=$(mktemp --suffix .setQtEnvironment.sh)
    echo "shellHook: setQtEnvironment = $setQtEnvironment"
    makeQtWrapper "/bin/sh" "$setQtEnvironment" "''${qtWrapperArgs[@]}"
    sed "/^exec/d" -i "$setQtEnvironment"
    source "$setQtEnvironment"
  '';
}
