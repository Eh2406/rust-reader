#[cfg(windows)]
extern crate winresource;

#[cfg(windows)]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set_manifest_file("rust_reader.manifest");
    res.set_icon_with_id("Reader2.ico", "1");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {
}