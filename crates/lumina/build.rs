
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../res/icons/lumina.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref()
        == Ok("windows")
    {
        let mut res = winresource::WindowsResource::new();

        res.set_icon("../../res/icons/lumina.ico");

        // Windows executable properties
        res.set("FileDescription", "Lumina Presentation");
        res.set("ProductName", "Lumina");
        res.set("InternalName", "lumina");
        res.set("OriginalFilename", "lumina.exe");

        res.compile()?;
    }

    Ok(())
}
