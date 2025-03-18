extern crate winres;
use winres::VersionInfo;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }

    let mut res = winres::WindowsResource::new();

    let mut version = 0;
    version |= 1 << 48;
    version |= 0 << 32;
    version |= 0 << 16;
    version |= 1;

    res.set_version_info(VersionInfo::FILEVERSION, version)
        .set_version_info(VersionInfo::PRODUCTVERSION, version);

    res.set_icon("./123.ico");

    if let Err(e) = res.compile() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
