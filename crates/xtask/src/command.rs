use anyhow::{Context, Result, ensure};
use std::process::Command;

pub fn run(command: &mut Command, description: &str) -> Result<()> {
    println!("\n==> {description}");
    println!("    {command:?}");

    let status = command.status().with_context(|| format!("Could not start {description}"))?;
    ensure!(status.success(), "{description} failed: {status}");
    Ok(())
}

#[cfg(windows)]
pub fn on_path(exe: &str) -> bool {
    Command::new("where.exe")
        .arg(exe)
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(not(windows))]
pub fn on_path(_exe: &str) -> bool {
    false
}
