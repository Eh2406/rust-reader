// Don't show a cmd if using nightly. This will be stabilized in 1.18.
// If using a stable before 1.18 build with for gnu `cargo rustc --release -- -C link-args=-mwindows`
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

fn read(voice: &mut SpVoice, list: &[RegexCleanerPair]) {
    voice.resume();
    match get_text() {
        Ok(x) => voice.speak(clean_text::<WideString>(&x, list)),
        Err(x) => {
            voice.speak_wait("oops. error.");
            println!("{:?}", x);
        }
    }
}

fn reload_settings(voice: &mut SpVoice, settings: &mut Settings, hk: &mut Vec<HotKey>) {
    let mut speech = String::new();
    if settings.reload_from_file() {
        hk.clear();
        *hk = setup_hotkeys(settings);
        settings.rate = voice.set_rate(settings.rate);
        settings.to_file();
        speech += "reloaded settings.\r\n";
    } else {
        speech += "failed to reload settings.\r\n";
    }
    speech += &make_speech(&settings, &hk);
    voice.speak(speech);
}

fn open_settings(settings: &mut Settings) {
    use std::process::Command;
    println!("{:?}",
             Command::new(r"C:\Windows\System32\notepad.exe")
                 .arg(settings.get_dir())
                 .spawn());
}

fn play_pause(voice: &mut SpVoice) {
    match voice.get_status().dwRunningState {
        2 => voice.pause(),
        _ => voice.resume(),
    }
}

fn rate_down(voice: &mut SpVoice, settings: &mut Settings) {
    settings.rate = voice.change_rate(-1);
    settings.to_file();
    println!("rate :{:?}", settings.rate);
}

fn rate_up(voice: &mut SpVoice, settings: &mut Settings) {
    settings.rate = voice.change_rate(1);
    settings.to_file();
    println!("rate :{:?}", settings.rate);
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
    let mut hk = setup_hotkeys(&mut settings);
    clipboard_setup();

    voice.speak(make_speech(&settings, &hk));

    while let Some(msg) = get_message() {
        match msg.message {
            winapi::WM_HOTKEY => {
                match msg.wParam { // match on generated HotKey id
                    0 => read(&mut voice, &settings.cleaners),
                    1 => close(),
                    2 => reload_settings(&mut voice, &mut settings, &mut hk),
                    3 => open_settings(&mut settings),
                    4 => {
                        voice.toggle_window_visible();
                    }
                    5 => play_pause(&mut voice),
                    6 => rate_down(&mut voice, &mut settings),
                    7 => rate_up(&mut voice, &mut settings),
                    _ => println!("unknown hot {}", msg.wParam),
                }
            }
            _ => {
                // println!("{:?}", msg);
                unsafe {
                    user32::TranslateMessage(&msg);
                    user32::DispatchMessageW(&msg);
                }
            }
        }
    }
    voice.resume();
    voice.speak_wait("bye!");
    settings.to_file();
}
