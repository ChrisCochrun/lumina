#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use lumina::run;
use miette::{IntoDiagnostic, Result};
mod gstreamer_runtime;
use gstreamer_runtime::configure;

fn main() -> Result<()> {
    configure().into_diagnostic()?;
    run()
}
