extern crate winapi;
extern crate ole32;
extern crate user32;
extern crate clipboard_win;
extern crate rustc_serialize; //To write rust objects to json

use std::ptr;
use std::mem;

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

fn print_voice(voice: &mut SpVoice, settings: &mut Settings) {
    voice.set_volume(99);
    println!("volume :{:?}", voice.get_volume());
    voice.set_rate(settings.rate);
    println!("rate :{:?}", voice.get_rate());
    voice.set_alert_boundary(winapi::SPEI_PHONEME);
    println!("alert_boundary :{:?}", voice.get_alert_boundary());
}

fn read(voice: &mut SpVoice) {
    voice.resume();
    match get_text() {
        Ok(x) => voice.speak(clean_text(&x)),
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
    settings.rate = voice.get_rate() - 1;
    voice.set_rate(settings.rate);
    println!("rate :{:?}", settings.rate);
}

fn rate_up(voice: &mut SpVoice, settings: &mut Settings) {
    settings.rate = voice.get_rate() + 1;
    voice.set_rate(settings.rate);
    println!("rate :{:?}", settings.rate);
}

fn close() {
    unsafe { user32::PostQuitMessage(0) }
}

fn main() {
    let _com = Com::new();
    let mut voice = SpVoice::new();
    let mut settings = Settings::from_file();
    print_voice(&mut voice, &mut settings);
    let _hk: Vec<_> = ([(2, 191), // ctrl-? key
                        (7, winapi::VK_ESCAPE as u32), // ctrl-alt-shift-esk
                        (7, 191), // ctrl-alt-shift-?
                        (2, winapi::VK_OEM_PERIOD as u32), // ctrl-.
                        (3, winapi::VK_OEM_MINUS as u32), // ctrl-alt--
                        (3, winapi::VK_OEM_PLUS as u32) /* ctrl-alt-= */])
                          .into_iter()
                          .enumerate() // generate HotKey id
                          .map(|(id, &(modifiers, vk))| {
                              HotKey::new(modifiers, vk, id as i32).unwrap() // make HotKey
                          })
                          .collect();

    voice.speak_wait("Ready!");
    let mut msg: winapi::MSG = unsafe { mem::zeroed() };
    while unsafe { user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } > 0 {
        match msg.message {
            winapi::WM_HOTKEY => {
                match msg.wParam { // match on generated HotKey id
                    0 => read(&mut voice),
                    1 => close(),
                    2 => println!("dwRunningState {}", voice.get_status().dwRunningState),
                    3 => play_pause(&mut voice),
                    4 => rate_down(&mut voice, &mut settings),
                    5 => rate_up(&mut voice, &mut settings),
                    _ => println!("unknown hot {}", msg.wParam),
                }
            }
            winapi::WM_QUERYENDSESSION => close(),
            winapi::WM_ENDSESSION => close(),
            _ => {
                println!("{:?}", msg);
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
