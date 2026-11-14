use crate::{GStreamerOptions, command, gstreamer};
use anyhow::{Result, ensure};
use std::{env, path::Path, process::Command};

pub fn doctor(args: &GStreamerOptions) -> Result<()> {
    println!("Lumina Windows build doctor");
    for tool in ["cargo.exe", "cmake.exe", "ninja.exe", "cl.exe", "link.exe"] {
        let found = command::on_path(tool);
        println!("  {} {tool}", if found { "[OK]" } else { "[MISSING]" });
    }

    match gstreamer::discover(&args.root) {
        Ok(root) => {
            gstreamer::check_source(&root, args.expected_version.as_deref())?;
            println!("  [OK] GStreamer MSVC x64");
        }
        Err(e) => println!("  [MISSING] GStreamer: {e}"),
    }

    let clang = env::var_os("LIBCLANG_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Program Files\LLVM\bin"));
    let clang_present = clang.join("libclang.dll").is_file();
    println!("  {} libclang.dll ({})", if clang_present { "[OK]" } else { "[MISSING]" }, clang.display());

    if env::var_os("SQLX_OFFLINE").is_none() && env::var_os("DATABASE_URL").is_none() {
        println!("  [WARN] Set SQLX_OFFLINE=true (with prepared metadata) or DATABASE_URL=sqlite://test.db");
    }
    Ok(())
}

pub fn package(
    workspace: &Path,
    args: &GStreamerOptions,
    skip_build: bool,
    debug: bool,
    msvc_toolset: &str,
) -> Result<()> {
    for tool in ["cargo.exe", "cmake.exe", "ninja.exe", "cl.exe"] {
        ensure!(command::on_path(tool), "Missing {tool} on PATH. Initialize a Visual Studio x64 developer environment first.");
    }
    let gst_root = gstreamer::discover(&args.root)?;
    gstreamer::check_source(&gst_root, args.expected_version.as_deref())?;
    gstreamer::stage(workspace, &gst_root)?;
    gstreamer::verify(workspace)?;

    let package_root = workspace.join("xtask");
    let config = package_root.join("Packager.windows.toml");
    ensure!(config.is_file(), "Missing packager config: {}", config.display());

    if !skip_build {
        let mut cmd = Command::new("cargo");
        cmd.args(["build", "-p", "lumina", "--locked"])
            .current_dir(workspace);
        if !debug { cmd.arg("--release"); }
        configure_build_env(&mut cmd, &gst_root, msvc_toolset);
        command::run(&mut cmd, "Build Lumina")?;
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("packager")
        .arg("--config")
        .arg("Packager.windows.toml")
        .current_dir(&package_root);
    if !debug { cmd.arg("--release"); }
    configure_build_env(&mut cmd, &gst_root, msvc_toolset);
    command::run(&mut cmd, "Package Lumina NSIS installer")?;
    println!("Windows packaging completed. Inspect generated NSIS artifact and verify its signature.");
    Ok(())
}

fn configure_build_env(command: &mut Command, gst_root: &Path, msvc_toolset: &str) {
    command.env("GSTREAMER_1_0_ROOT_MSVC_X86_64", gst_root);
    if env::var_os("CMAKE_GENERATOR").is_none() {
        command.env("CMAKE_GENERATOR", "Ninja");
    }
    if env::var_os("MUPDF_MSVC_PLATFORM_TOOLSET").is_none() {
        command.env("MUPDF_MSVC_PLATFORM_TOOLSET", msvc_toolset);
    }
    if env::var_os("LIBCLANG_PATH").is_none() {
        let fallback = Path::new(r"C:\Program Files\LLVM\bin");
        if fallback.join("libclang.dll").is_file() {
            command.env("LIBCLANG_PATH", fallback);
        }
    }
}
