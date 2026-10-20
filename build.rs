//! Bäddar in ikonen i Windows-programmet (syns i Utforskaren och på genvägar).

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "Modellverkstan");
        res.set("ProductName", "Modellverkstan");
        if let Err(e) = res.compile() {
            println!("cargo:warning=Kunde inte bädda in ikonen: {e}");
        }
    }
}
