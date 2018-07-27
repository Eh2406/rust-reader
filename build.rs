#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("rust_reader.manifest");
    res.set_icon("Reader2.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {
}