mod command;
mod gstreamer;
mod windows;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Lumina build and release tooling", version)]
struct Cli {
    #[command(subcommand)]
    command: Tasks,
}

#[derive(Debug, Subcommand)]
enum Tasks {
    /// Check the Windows toolchain and GStreamer installation.
    Doctor(GStreamerOptions),
    /// Prepare or verify the private GStreamer runtime.
    #[command(subcommand)]
    Gstreamer(GStreamerTasks),
    /// Build and package Lumina.
    #[command(subcommand)]
    Package(PackageTasks),
}

#[derive(Debug, Args, Clone)]
struct GStreamerOptions {
    /// Directory containing bin/gstreamer-1.0-0.dll.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Require gst-inspect --version to contain this string, e.g. 1.28.8.
    #[arg(long)]
    expected_version: Option<String>,
}

#[derive(Debug, Subcommand)]
enum GStreamerTasks {
    /// Copy the GStreamer runtime into the app's packaging directory.
    Stage(GStreamerOptions),
    /// Check that the staged runtime is complete enough to package.
    Verify,
}

#[derive(Debug, Subcommand)]
enum PackageTasks {
    /// Stage GStreamer, build Lumina, and produce an NSIS installer.
    Windows {
        #[command(flatten)]
        gst: GStreamerOptions,
        /// Explicitly select the release profile (also the default).
        #[arg(long, conflicts_with = "debug")]
        release: bool,
        /// Skip Cargo compilation (requires an existing matching binary).
        #[arg(long)]
        skip_build: bool,
        /// Package the debug build instead of the release build.
        #[arg(long)]
        debug: bool,
        /// Platform toolset family for MuPDF (VS 2026 = v145; VS 2022 = v143).
        #[arg(long, default_value = "v145")]
        msvc_toolset: String,
    },
}

fn main() -> Result<()> {
    if !cfg!(windows) {
        bail!("These xtask commands must run on a Windows build machine");
    }

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| anyhow::anyhow!("xtask must live directly under the workspace root"))?
        .to_path_buf();

    match Cli::parse().command {
        Tasks::Doctor(args) => windows::doctor(&args),
        Tasks::Gstreamer(GStreamerTasks::Stage(args)) => {
            let gst_root = gstreamer::discover(&args.root)?;
            gstreamer::check_source(&gst_root, args.expected_version.as_deref())?;
            gstreamer::stage(&workspace, &gst_root)?;
            gstreamer::verify(&workspace)
        }
        Tasks::Gstreamer(GStreamerTasks::Verify) => gstreamer::verify(&workspace),
        Tasks::Package(PackageTasks::Windows {
            gst,
            release: _,
            skip_build,
            debug,
            msvc_toolset,
        }) => windows::package(&workspace, &gst, skip_build, debug, &msvc_toolset),
    }
}
