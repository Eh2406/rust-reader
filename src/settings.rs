use rustc_serialize::json;
use winapi::{VK_OEM_2, VK_ESCAPE, VK_OEM_PERIOD, VK_OEM_MINUS, VK_OEM_PLUS};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::env::current_exe;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Settings {
    pub rate: i32,
    pub hotkeys: [(u32, u32); 6],
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            rate: 6,
            hotkeys: [(2, VK_OEM_2 as u32), // ctrl-? key
                      (7, VK_ESCAPE as u32), // ctrl-alt-shift-esk
                      (7, VK_OEM_2 as u32), // ctrl-alt-shift-?
                      (2, VK_OEM_PERIOD as u32), // ctrl-.
                      (3, VK_OEM_MINUS as u32), // ctrl-alt--
                      (3, VK_OEM_PLUS as u32) /* ctrl-alt-= */],
        }
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
    pub fn to_file(&self) {
        File::create(Settings::path())
            .map(|mut f| write!(f, "{}", json::as_pretty_json(&self)).unwrap_or(()))
            .unwrap_or(());
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        self.to_file()
    }
}
