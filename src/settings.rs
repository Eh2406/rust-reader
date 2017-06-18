use preferences::{Preferences, AppInfo, prefs_base_dir};
use winapi::{VK_OEM_2, VK_ESCAPE, VK_OEM_PERIOD, VK_OEM_MINUS, VK_OEM_PLUS};
use clean_text::RegexCleanerPair;

const APP_INFO: AppInfo = AppInfo {
    name: "rust_reader",
    author: "us",
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub rate: i32,
    pub hotkeys: [(u32, u32); 8],
    pub cleaners: Vec<RegexCleanerPair>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            rate: 6,
            hotkeys: [
                (2, VK_OEM_2 as u32), // ctrl-? key
                (7, VK_ESCAPE as u32), // ctrl-alt-shift-esk
                (7, 0x52 as u32), // ctrl-alt-shift-r
                (7, 0x53 as u32), // ctrl-alt-shift-s
                (3, VK_OEM_2 as u32), // ctrl-alt-?
                (2, VK_OEM_PERIOD as u32), // ctrl-.
                (3, VK_OEM_MINUS as u32), // ctrl-alt--
                (3, VK_OEM_PLUS as u32), // ctrl-alt-=
            ],
            cleaners: RegexCleanerPair::prep_list(
                &[
                    (r"\s+", " "),
                    (
                        concat!(
                            r"(https?://)?(?P<a>[-a-zA-Z0-9@:%._\+~#=]{2,256}",
                            r"\.[a-z]{2,6})\b[-a-zA-Z0-9@:%_\+.~#?&//=]{10,}"
                        ),
                        "link to $a",
                    ),
                    (
                        r"(?P<s>[0-9a-f]{6})([0-9]+[a-f]|[a-f]+[0-9])[0-9a-f]*",
                        "hash $s",
                    ),
                ],
            ).unwrap(),
        }
    }
    pub fn get_dir(&self) -> ::std::path::PathBuf {
        prefs_base_dir()
            .map(|mut p| {
                p.push("us");
                p.push("rust_reader");
                p.push("setings.prefs.json");
                p
            })
            .unwrap_or_default()
    }
    pub fn from_file() -> Settings {
        Settings::load(&APP_INFO, "setings").unwrap_or_else(|_| {
            println!("failed to lode settings.");
            Settings::new()
        })
    }
    pub fn reload_from_file(&mut self) -> bool {
        if let Ok(new) = Settings::load(&APP_INFO, "setings") {
            println!("reload settings.");
            *self = new;
            true
        } else {
            println!("failed to reload settings.");
            false
        }
    }
    pub fn to_file(&self) {
        if self.save(&APP_INFO, "setings").is_err() {
            println!("failed to save settings.");
        }
    }
}
