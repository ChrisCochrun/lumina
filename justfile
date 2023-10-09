default:
    just --list
build:
    cmake -DCMAKE_BUILD_TYPE=Release -B bld/ .
    make -j8 --dir bld/
    rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/
