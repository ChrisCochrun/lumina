;; This file is provided to assist in setting up a development
;; environment for Lumina.
;; 
;; Author: Chris Cochrun
;; 
;; 
;; 
;; 
;; 
;; 
;; 
;; 
;; 
;; 

(use-modules (gnu packages)
             (gnu packages rust)
             (gnu packages rust-apps)
             (gnu packages llvm)
             (gnu packages qt)
             (gnu packages gdb)
             (gnu packages mold)
             (gnu packages pkg-config)
             (gnu packages kde-frameworks)
             (gnu packages kde)
             (gnu packages video)
             (gnu packages cmake)
             (gnu packages crates-io)
             (gnu services)
             (guix gexp)
             (guix packages)
             (guix git-download)
             (guix build-system qt)
             (guix build-system cmake)
             ((guix licenses) #:prefix license:))

(define this-directory
  (dirname (local-file-absolute-file-name (local-file "guix.scm"))))

(define source
  (local-file this-directory
              #:recursive? #t
              #:select? (git-predicate this-directory)))

;; Corrosion allows us to build a cmake project that uses rust too.
(define-public corrosion
  (let ((commit "6ae04cf691fa721945428b2f96b0818085135890")
        (revision "0.4.1"))
    (package
      (name "corrosion")
      (version (git-version "0.4.1" revision commit))
      (source
       (origin
         (method git-fetch)
         (uri (git-reference
               (url "https://github.com/corrosion-rs/corrosion.git")
               (commit commit)))
         (file-name (git-file-name name version))
         (sha256
          (base32
           "1f0zmqm0iz669zqv74222x9759jbn1zq5z4snjwkd5g3lv0p4mkw"))))
      (build-system cmake-build-system)
      (arguments `(#:tests? #f))
      (inputs (list
               cmake
               `(,rust "out")
               `(,rust "cargo")))
      (license license:gpl3+)
      (home-page "idk")
      (synopsis "Adding rust to cmake projects")
      (description "idk"))))

(define-public qtfull
  (package
    (inherit qtbase-5)
    (propagated-inputs (modify-inputs
                        (package-inputs qtbase-5)
                        (append
                         qtdeclarative-5
                         qtquickcontrols2-5
                         qtx11extras
                         qtwayland-5
                         qtwebengine-5
                         qttools-5)))))

;; (define-public rust-cxx-qt-1
;;   (package
;;     (name "rust-cxx")
;;     (version "1.0.86")
;;     (source
;;       (origin
;;         (method url-fetch)
;;         (uri (crate-uri "cxx" version))
;;         (file-name
;;          (string-append name "-" version ".tar.gz"))
;;         (sha256
;;          (base32
;;           "0yc5gz723hiwqk7waygj63655fh5vzq3551p1j2wyzc06xf0glai"))))
;;     (build-system cargo-build-system)
;;     (arguments
;;      `(#:tests? #f  ; Cannot compile cxx-test-suite.
;;        #:cargo-inputs
;;        (("rust-cc" ,rust-cc-1)
;;         ("rust-cxxbridge-flags" ,rust-cxxbridge-flags-1)
;;         ("rust-cxxbridge-macro" ,rust-cxxbridge-macro-1)
;;         ("rust-link-cplusplus" ,rust-link-cplusplus-1))
;;        #:cargo-development-inputs
;;        (("rust-cxx-build" ,rust-cxx-build-1)
;;         ("rust-cxx-gen" ,rust-cxx-gen-0.7)
;;         ("rust-cxx-test-suite" ,rust-cxx-test-suite-0.0.0)
;;         ("rust-rustversion" ,rust-rustversion-1)
;;         ("rust-trybuild" ,rust-trybuild-1))))
;;     (home-page "https://cxx.rs")
;;     (synopsis "Safe interop between Rust and C++")
;;     (description "This package provides a safe interop between Rust and C++.")
;;     (license (list license:expat license:asl2.0))))


(define-public lumina
  (package
    (name "lumina")
    (version "0.0.1")
    (source source)
    (build-system qt-build-system)
    (arguments `(#:phases
                 (modify-phases %standard-phases
                   (replace 'build
                     (lambda* (#:key outputs #:allow-other-keys)
                       (invoke "/bin/sh" "./build.sh" "-d"))))))

    (inputs (list mpv
                  ffmpeg))
    (propagated-inputs (list clang
                             cmake
                             mold
                             clazy
                             clang-toolchain
                             mold
                             gdb
                             pkg-config
                             qtfull
                             qttools-5
                             qt-creator
                             qtdeclarative-5
                             qtquickcontrols2-5
                             qtx11extras
                             qtwayland-5
                             qtwebengine-5
                             kirigami
                             kirigami-addons
                             qqc2-desktop-style
                             extra-cmake-modules
                             karchive
                             kcoreaddons
                             ki18n
                             sonnet
                             ;; corrosion is needed for build and is yet to
                             ;; be packaged.
                             corrosion

                             `(,rust "out")
                             `(,rust "rustfmt")
                             `(,rust "cargo")
                             ;; rust-analyzer
                             rust-clippy-0.0))

    (native-search-paths
     (list (search-path-specification
            (variable "CMAKE_INCLUDE_PATH")
            (files '("include")))))

    (license license:gpl3+)
    (home-page "idk")
    (synopsis "A Church Presentation Application")
    (description "idk")))

lumina
