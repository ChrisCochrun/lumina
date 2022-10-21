{ pkgs ? import <nixpkgs> { } }:
with pkgs;
# {
#   stdenv,
#   lib,
#   # kglobalaccel,
#   # kinit,
#   # kwin,
#   # kio,
#   # kguiaddons,
#   # kcoreaddons,
#   gcc,
#   gnumake,
#   clang,
#   cmake,
#   extra-cmake-modules,
#   pkg-config,
#   wrapQtAppsHook,
#   qtbase,
#   qt5Full,
#   clang-tools,
#   qttools,
#   qtquickcontrols2,
#   qtx11extras,
#   qtmultimedia,
#   kirigami2,
#   ki18n,
#   kcoreaddons,
#   # lightly-qt,
#   mpv
# }:

stdenv.mkDerivation rec {
  name = "Libre Presenter";
  pname = "libre-presenter";
  version = "0.0.0";

  src = ./.;

  nativeBuildInputs = [
    gcc
    gnumake
    clang
    clang-tools
    cmake
    extra-cmake-modules
    pkg-config
    libsForQt5.wrapQtAppsHook
    # gccStdenv
    # stdenv
  ];

  buildInputs = [
    libsForQt5.qt5.full
    libsForQt5.qttools
    libsForQt5.qtquickcontrols2
    libsForQt5.qtx11extras
    libsForQt5.qtmultimedia
    # qtwayland
    libsForQt5.kirigami2
    # breeze-icons
    # breeze-qt5
    # qqc2-desktop-style
    libsForQt5.karchive
    libsForQt5.ki18n
    libsForQt5.kcoreaddons
    # lightly-qt
    podofo
    mpv
    # libsForQt5.kconfig
    # ffmpeg-full
    # yt-dlp
  ];

  # preConfigure = ''
  #   # local modulepath=$(kf5-config --install module)
  #   # local datapath=$(kf5-config --install data)
  #   # local servicespath=$(kf5-config --install services)
  #   # substituteInPlace CMakeLists.txt \
  #   #   --replace "\''${MODULEPATH}" "$out/''${modulepath#/nix/store/*/}" \
  #   #   --replace "\''${DATAPATH}"   "$out/''${datapath#/nix/store/*/}"

  #   # substituteInPlace CMakeLists.txt \
  #   #   --replace "\''${MODULEPATH}" "$out/qt-5.15.3/plugins" \
  #   #   --replace "\''${DATAPATH}"   "$out/share"
  # '';

  # postConfigure = ''
  #   substituteInPlace cmake_install.cmake \
  #     --replace "${kdelibs4support}" "$out"

  # '';

  configurePhase = ''
  cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=1 -B build/ .
  '';

  buildPhase = ''
  make --dir build/
  rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/
  '';

  installPhase = ''
  mkdir -p $out/bin
  mv build/bin/presenter $out/bin
  '';

  meta = with lib; {
    name = "Libre Presenter";
    description = "A church presentation software made with QT/QML";
    homepage = "";
    license = licenses.gpl3;
    maintainers = [ "chriscochrun" ];
    platforms = platforms.all;
  };
}
