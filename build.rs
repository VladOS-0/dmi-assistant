use std::{env, io};

use winresource::WindowsResource;

pub fn main() -> io::Result<()> {
    println!("cargo::rerun-if-changed=assets/fonts/fontello.toml");
    iced_fontello::build("assets/fonts/fontello.toml")
        .expect("Build fontello font");
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("assets/images/icon.ico")
            .set_language(0x0009) // English
            .set("ProductName", "DMI Assistant")
            .set("OriginalFilename", "DMI Assistant.exe")
            .set(
                "FileDescription",
                "Simple GUI application for viewing DreamMaker Icon files.",
            )
            .set("LegalCopyright", "Copyleft ɔ Vlad0s")
            .compile()
            .expect("Building winresource");
    }
    Ok(())
}
