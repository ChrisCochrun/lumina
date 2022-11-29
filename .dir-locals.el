;;; Directory Local Variables
;;; For more information see (info "(emacs) Directory Variables")

((nil . ((projectile-project-compilation-cmd . "cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=1 -B build/ . && make -j8 --dir bld/ && rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/")
         (compile-command . "cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=1 -B build/ . && make -j8 --dir bld/ && rm -rf ~/.cache/librepresenter/Libre\ Presenter/qmlcache/")
         (projectile-project-run-cmd . "./bld/bin/presenter")))
 (c++-mode . ((aggressive-indent-mode . nil))))
