default:
    just --list
build:
    cmake -DCMAKE_BUILD_TYPE=Debug -B bld/ .
    make -j8 --dir bld/
    rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/
