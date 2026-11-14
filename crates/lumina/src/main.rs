use lumina::run;
use miette::{IntoDiagnostic, Result};
mod gstreamer_runtime;
use gstreamer_runtime::configure;

fn main() -> Result<()> {
    configure().into_diagnostic()?;
    run()
}
