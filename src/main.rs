// Dont show a cmd if using nightly. This will be stabalized in 1.18.
// If using a stable befor 1.18 build with `cargo rustc --release -- -C link-args=-mwindows`
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
extern crate preferences; //save objects in appdata folder
extern crate regex;
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

#[allow(dead_code)]
mod clean_text;
use clean_text::*;

fn read(voice: &mut SpVoice) {
    voice.resume();
    match get_text() {
        Ok(x) => voice.speak(clean_text(x, &RE_LIST)),
        Err(x) => {
            voice.speak_wait("oops. error.");
            println!("{:?}", x);
        }
    }
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

fn main() {
    let _com = Com::new();
    let mut voice = SpVoice::new();
    let mut settings = Settings::from_file();
    voice.set_rate(settings.rate);
    println!("rate :{:?}", voice.get_rate());
    let hk: Vec<_> = settings.hotkeys
                             .into_iter()
                             .enumerate() // generate HotKey id
                             .map(|(id, &(modifiers, vk))| {
                                 HotKey::new(modifiers, vk, id as i32).unwrap() // make HotKey
                             })
                             .collect();
    clipboard_setup();

    let mut setup_spech = "Reading from settings at:\r\n".to_string();
    setup_spech += &settings.get_dir().to_string_lossy();
    setup_spech += "\r\n";
    setup_spech += "speech rate of: ";
    setup_spech += &settings.rate.to_string();
    setup_spech += "\r\n";
    setup_spech += "hotkeys\r\n";
    for (t, h) in ["read",
                   "close",
                   "toggle_window_visible",
                   "play_pause",
                   "rate_down",
                   "rate_up"]
                .iter()
                .zip(hk.iter()) {
        setup_spech += t;
        setup_spech += ": ";
        setup_spech += &h.display();
        setup_spech += "\r\n";
    }
    setup_spech += "Ready!";
    voice.speak(setup_spech);

    while let Some(msg) = get_message() {
        match msg.message {
            winapi::WM_HOTKEY => {
                match msg.wParam { // match on generated HotKey id
                    0 => read(&mut voice),
                    1 => close(),
                    2 => {
                        voice.toggle_window_visible();
                    }
                    3 => play_pause(&mut voice),
                    4 => rate_down(&mut voice, &mut settings),
                    5 => rate_up(&mut voice, &mut settings),
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
}
