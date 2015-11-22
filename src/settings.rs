use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::env::current_exe;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Settings {
    pub rate: i32,
}

impl Settings {
    pub fn new() -> Settings {
        Settings { rate: 6 }
    }
    pub fn path() -> PathBuf {
        let mut path = current_exe().unwrap();
        path.set_extension("json");
        path
    }
    pub fn from_file() -> Settings {
        File::open(Settings::path())
            .map(|mut f| {
                let mut s = String::new();
                f.read_to_string(&mut s)
                 .map(|_| json::decode(&s).unwrap_or(Settings::new()))
                 .unwrap_or(Settings::new())
            })
            .unwrap_or(Settings::new())
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        json::encode(self)
            .map(|s| {
                File::create(Settings::path())
                    .map(|mut f| f.write_all(s.as_bytes()).unwrap_or(()))
                    .unwrap_or(())
            })
            .unwrap_or(());
    }
}
