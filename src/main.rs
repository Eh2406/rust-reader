// Don't show a cmd if using nightly. This will be stabilized in 1.18.
// If using a stable before 1.18 build with
// for gnu `cargo rustc --release -- -C link-args=-mwindows`
#![cfg_attr(feature="nightly", windows_subsystem = "windows")]

extern crate winapi;
extern crate ole32;
extern crate user32;
extern crate kernel32;
extern crate clipboard_win;
extern crate unicode_segmentation;

#[macro_use]
extern crate serde_derive; //To write rust objects to json
extern crate serde;
extern crate preferences; //save objects in app data folder
extern crate regex;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
extern crate quickcheck;

mod wide_string;
mod window;
use window::*;

mod sapi;
use sapi::*;

mod clipboard;
use clipboard::*;

mod hot_key;
use hot_key::*;

mod settings;
use settings::*;

mod clean_text;
use clean_text::*;

#[derive(Debug)]
struct State<'a> {
    voice: Box<SpVoice<'a>>,
    settings: Settings,
    hk: Vec<HotKey>,
}

impl<'a> State<'a> {
    fn read(&mut self) {
        self.voice.resume();
        match get_text() {
            Ok(x) => self.voice.speak(clean_text::<WideString>(&x, &self.settings.cleaners)),
            Err(x) => {
                self.voice.speak("oops. error.");
                println!("{:?}", x);
            }
        }
    }

    fn reload_settings(&mut self) {
        let mut speech = String::new();
        if self.settings.reload_from_file() {
            self.hk.clear();
            self.hk = setup_hotkeys(&mut self.settings);
            self.settings.rate = self.voice.set_rate(self.settings.rate);
            self.settings.to_file();
            speech += "reloaded settings.\r\n";
        } else {
            speech += "failed to reload settings.\r\n";
        }
        speech += &make_speech(&self.settings, &self.hk);
        self.voice.resume();
        self.voice.speak(speech);
    }

    fn open_settings(&self) {
        use std::process::Command;
        println!("{:?}",
                Command::new(r"C:\Windows\System32\notepad.exe")
                        .arg(self.settings.get_dir())
                        .spawn());
    }

    fn toggle_window_visible(&mut self) {
        self.voice.toggle_window_visible();
    }

    fn play_pause(&mut self) {
        match self.voice.get_status().dwRunningState {
            2 => self.voice.pause(),
            _ => self.voice.resume(),
        }
    }

    fn rate_down(&mut self) {
        self.settings.rate = self.voice.change_rate(-1);
        self.settings.to_file();
        println!("rate :{:?}", self.settings.rate);
    }

    fn rate_up(&mut self) {
        self.settings.rate = self.voice.change_rate(1);
        self.settings.to_file();
        println!("rate :{:?}", self.settings.rate);
    }
}

fn setup_hotkeys(settings: &mut Settings) -> Vec<HotKey> {
    settings.hotkeys
            .into_iter()
            .enumerate() // generate HotKey id
            .map(|(id, &(modifiers, vk))| {
                HotKey::new(modifiers, vk, id as i32).unwrap() // make HotKey
            })
            .collect()
}

fn press_hotkey(id: winapi::WPARAM) {
    unsafe {
            user32::PostThreadMessageW(kernel32::GetCurrentThreadId(),
                                        winapi::WM_HOTKEY,
                                        id,
                                        0)};
}

impl<'a> State<'a> {
    fn match_hotkey_id(&mut self, id: winapi::WPARAM) {
        match id { // match on generated HotKey id
            0 => self.read(),
            1 => close(),
            2 => self.reload_settings(),
            3 => self.open_settings(),
            4 => self.toggle_window_visible(),
            5 => self.play_pause(),
            6 => self.rate_down(),
            7 => self.rate_up(),
            _ => println!("unknown hot {}", id),
        }
    }
}

fn make_speech(settings: &Settings, hk: &[HotKey]) -> String {
    let mut out = "Reading from settings at:".to_string();
    out += "\r\n";
    out += &settings.get_dir().to_string_lossy();
    out += "\r\n";
    out += "speech rate of: ";
    out += &settings.rate.to_string();
    out += "\r\n";
    out += "hotkeys\r\n";
    for (t, h) in ["read",
                   "close",
                   "reload_settings",
                   "open_settings",
                   "toggle_window_visible",
                   "play_pause",
                   "rate_down",
                   "rate_up"]
                .iter()
                .zip(hk.iter()) {
        out += t;
        out += ": ";
        out += &h.display();
        out += "\r\n";
    }
    out += "Ready!";
    out
}

fn main() {
    let com = Com::new();
    let mut voice = SpVoice::new(&com);
    let mut settings = Settings::from_file();
    voice.set_rate(settings.rate);
    let hk = setup_hotkeys(&mut settings);
    clipboard_setup();

    let mut state = State {
        voice: voice,
        settings: settings,
        hk: hk,
    };

    state.voice.speak(make_speech(&state.settings, &state.hk));

    while let Some(msg) = get_message() {
        match msg.message {
            winapi::WM_HOTKEY => state.match_hotkey_id(msg.wParam),
            _ => {
                // println!("{:?}", msg);
                unsafe {
                    user32::TranslateMessage(&msg);
                    user32::DispatchMessageW(&msg);
                }
            }
        }
    }
    state.voice.resume();
    state.voice.speak_wait("bye!");
    state.settings.to_file();
}
