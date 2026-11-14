use anyhow::{Context, Result, bail, ensure};
use std::{env, ffi::OsStr, fs, path::{Path, PathBuf}, process::Command};

const REQUIRED_DLLS: &[&str] = &[
    "glib-2.0-0.dll",
    "gobject-2.0-0.dll",
    "gstreamer-1.0-0.dll",
    "gstbase-1.0-0.dll",
    "gstpbutils-1.0-0.dll",
];

const REQUIRED_PLUGINS: &[&str] = &["gstcoreelements.dll", "gstplayback.dll"];

pub fn discover(explicit: &Option<PathBuf>) -> Result<PathBuf> {
    let root = if let Some(root) = explicit {
        root.clone()
    } else if let Some(root) = env::var_os("GSTREAMER_1_0_ROOT_MSVC_X86_64") {
        PathBuf::from(root)
    } else {
        let program_files = env::var_os("ProgramFiles")
            .context("Neither --root, GSTREAMER_1_0_ROOT_MSVC_X86_64 nor ProgramFiles is set")?;
        PathBuf::from(program_files).join("gstreamer/1.0/msvc_x86_64")
    };
    ensure!(root.is_dir(), "GStreamer root not found: {}", root.display());
    Ok(root)
}

pub fn check_source(root: &Path, expected_version: Option<&str>) -> Result<()> {
    println!("GStreamer SDK: {}", root.display());
    for dll in REQUIRED_DLLS {
        require_file(&root.join("bin").join(dll))?;
    }
    let plugins = root.join("lib/gstreamer-1.0");
    ensure!(plugins.is_dir(), "GStreamer plugins directory missing: {}", plugins.display());
    for plugin in REQUIRED_PLUGINS {
        require_file(&plugins.join(plugin))?;
    }
    let _scanner = find_scanner(root)?;

    if let Some(version) = expected_version {
        let inspect = root.join("bin/gst-inspect-1.0.exe");
        require_file(&inspect)?;
        let output = Command::new(&inspect)
            .arg("--version")
            .output()
            .with_context(|| format!("Cannot run {}", inspect.display()))?;
        ensure!(output.status.success(), "gst-inspect --version failed");
        let actual = String::from_utf8_lossy(&output.stdout);
        ensure!(actual.contains(version), "Expected GStreamer {version}; got {actual}");
        print!("{actual}");
    }
    Ok(())
}

pub fn stage_dir(workspace: &Path) -> PathBuf {
    workspace.join("packaging/windows/gst")
}

pub fn stage(workspace: &Path, root: &Path) -> Result<()> {
    check_source(root, None)?;
    let dest = stage_dir(workspace);
    let temp = dest.with_extension("staging");
    // This is a known path inside Lumina, so create its parent on first run.
    fs::create_dir_all(dest.parent().context("Stage directory has no parent")?)?;

    if temp.exists() {
        fs::remove_dir_all(&temp).with_context(|| format!("Cannot clear {}", temp.display()))?;
    }
    fs::create_dir_all(&temp)?;
    let result = (|| -> Result<()> {
        let dll_count = copy_dlls(&root.join("bin"), &temp.join("dll"))?;
        let plugin_count = copy_dlls(&root.join("lib/gstreamer-1.0"), &temp.join("plugins"))?;
        let scanner = find_scanner(root)?;
        fs::copy(&scanner, temp.join("gst-plugin-scanner.exe"))
            .with_context(|| format!("Cannot copy scanner from {}", scanner.display()))?;
        verify_dir(&temp)?;
        let manifest = format!(
            "source={}\nruntime_dlls={}\nplugins={}\nscanner={}\n",
            root.display(), dll_count, plugin_count, scanner.display(),
        );
        fs::write(temp.join("stage-info.txt"), manifest)?;
        println!("Staged {dll_count} runtime DLLs, {plugin_count} plugins, and scanner");
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&temp);
    }
    result?;

    // Replace old staging only after a complete new copy passes validation.
    if dest.exists() {
        fs::remove_dir_all(&dest).with_context(|| format!("Cannot remove old stage {}", dest.display()))?;
    }
    fs::rename(&temp, &dest)
        .with_context(|| format!("Cannot move staging directory into {}", dest.display()))?;
    println!("Staged at: {}", dest.display());
    Ok(())
}

pub fn verify(workspace: &Path) -> Result<()> {
    let stage = stage_dir(workspace);
    verify_dir(&stage)?;
    println!("GStreamer staging validated: {}", stage.display());
    Ok(())
}

fn verify_dir(stage: &Path) -> Result<()> {
    for dll in REQUIRED_DLLS {
        require_file(&stage.join("dll").join(dll))?;
    }
    for plugin in REQUIRED_PLUGINS {
        require_file(&stage.join("plugins").join(plugin))?;
    }
    require_file(&stage.join("gst-plugin-scanner.exe"))?;
    Ok(())
}

fn find_scanner(root: &Path) -> Result<PathBuf> {
    let candidates = [
        root.join("libexec/gstreamer-1.0/gst-plugin-scanner.exe"),
        root.join("lib/gstreamer-1.0/gst-plugin-scanner.exe"),
        root.join("bin/gst-plugin-scanner.exe"),
    ];
    candidates.into_iter().find(|p| p.is_file()).ok_or_else(|| {
        anyhow::anyhow!("gst-plugin-scanner.exe not found under {}", root.display())
    })
}

fn require_file(file: &Path) -> Result<()> {
    ensure!(file.is_file(), "Required GStreamer file missing: {}", file.display());
    Ok(())
}

fn copy_dlls(source: &Path, destination: &Path) -> Result<usize> {
    ensure!(source.is_dir(), "Directory missing: {}", source.display());
    fs::create_dir_all(destination)?;
    let mut count = 0;
    for entry in fs::read_dir(source).with_context(|| format!("Cannot read {}", source.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_dll(&path) {
            let target = destination.join(entry.file_name());
            fs::copy(&path, &target)
                .with_context(|| format!("Cannot copy {} to {}", path.display(), target.display()))?;
            count += 1;
        }
    }
    if count == 0 { bail!("No DLLs found in {}", source.display()); }
    Ok(count)
}

fn is_dll(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dll_extension_is_case_insensitive() {
        assert!(is_dll(Path::new("foo.DLL")));
        assert!(is_dll(Path::new("foo.dll")));
        assert!(!is_dll(Path::new("foo.lib")));
    }
}
