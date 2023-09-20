# SPDX-FileCopyrightText: Chris Cochrun <chris@cochrun.xyz>
# SPDX-License-Identifier: MIT

MAKEFLAGS += --always-make

all: build

build:
	cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=1 -DCMAKE_BUILD_TYPE=Debug -B bld/ . && make -j8 --dir bld/ && rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/	

install: build
	scripts/install.sh

uninstall:
	scripts/uninstall.sh
docs:
	npx typedoc --out build/docs

test:
	scripts/test.sh
