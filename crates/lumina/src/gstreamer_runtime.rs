#[cfg(windows)]
pub fn configure() -> std::io::Result<()> {
    use std::{env, io, path::PathBuf};

    let exe: PathBuf = env::current_exe()?;
    let directory = exe.parent().ok_or_else(|| io::Error::other("Missing executable parent"))?;
    let plugins = directory.join("gstreamer").join("plugins");
    if !plugins.is_dir() {
        // Unbundled development build: use installed GStreamer SDK.
        return Ok(());
    }
    let scanner = directory.join("gst-plugin-scanner.exe");
    if !scanner.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Bundled plugin scanner is missing"));
    }

    // Call once at the start of main(), before any threads or gst::init().
    // See Rust 2024's safety requirements around process environment mutation.
    unsafe {
        env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &plugins);
        env::set_var("GST_PLUGIN_PATH_1_0", "");
        env::set_var("GST_PLUGIN_SCANNER_1_0", &scanner);
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn configure() -> std::io::Result<()> {
    Ok(())
}
