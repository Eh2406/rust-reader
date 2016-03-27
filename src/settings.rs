use preferences::Preferences;
use winapi::{VK_OEM_2, VK_ESCAPE, VK_OEM_PERIOD, VK_OEM_MINUS, VK_OEM_PLUS};

const SETTINGS_PATH: &'static str = "rust_reader/setings";

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
    pub fn from_file() -> Settings {
        let mut new = Settings::new();
        if new.load(SETTINGS_PATH).is_err() {
            println!("failed to lode settings.");
        }
        new
    }
    pub fn to_file(&self) {
        if self.save(SETTINGS_PATH).is_err() {
            println!("failed to save settings.");
        }
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        self.to_file()
    }
}
