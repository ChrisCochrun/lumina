{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell {
  name = "presenter-env";

  nativeBuildInputs = [
    gcc
    gnumake
    clang
    cmake
    extra-cmake-modules
    pkg-config
    libsForQt5.wrapQtAppsHook
    # gccStdenv
    # stdenv
  ];

  buildInputs = [
    clang-tools
    qt5.full
    qt5.qttools
    qt5.qtquickcontrols2
    qt5.qtx11extras
    qt5.qtmultimedia
    qt5.qtwayland
    libsForQt5.kirigami2
    libsForQt5.breeze-icons
    libsForQt5.breeze-qt5
    libsForQt5.qqc2-desktop-style
    # libsForQt5.kirigami-addons
    libsForQt5.ki18n
    libsForQt5.kcoreaddons

    # This is only here because apparently it doesn't pick up the icon theme from the base system
    papirus-icon-theme
    # lightly-qt

    mpv
    # libsForQt5.kconfig
    # ffmpeg-full
    # yt-dlp
  ];
  
  # This creates the proper qt env so that plugins are found right.
  shellHook = ''
    fish
  '';
}