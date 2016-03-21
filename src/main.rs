extern crate winapi;
extern crate ole32;
extern crate user32;
extern crate kernel32;
extern crate clipboard_win;
extern crate unicode_segmentation;
extern crate rustc_serialize; //To write rust objects to json

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

fn print_voice(voice: &mut SpVoice, settings: &mut Settings) {
    voice.set_volume(100);
    println!("volume :{:?}", voice.get_volume());
    voice.set_rate(settings.rate);
    println!("rate :{:?}", voice.get_rate());
    voice.set_alert_boundary(winapi::SPEI_PHONEME);
    println!("alert_boundary :{:?}", voice.get_alert_boundary());
}

fn read(voice: &mut SpVoice) {
    voice.resume();
    match get_text() {
        Ok(x) => voice.speak(clean_text(x)),
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
    print_voice(&mut voice, &mut settings);
    let _hk: Vec<_> = settings.hotkeys
                              .into_iter()
                              .enumerate() // generate HotKey id
                              .map(|(id, &(modifiers, vk))| {
                                  HotKey::new(modifiers, vk, id as i32).unwrap() // make HotKey
                              })
                              .collect();

    if clipboard_win::wrapper::get_clipboard_seq_num().is_none() {
        // this will crash on our reding so lets get it over with.
        // this may fix the problem
        clipboard_win::set_clipboard("").unwrap();
        // let us see if it did
        clipboard_win::wrapper::get_clipboard_seq_num()
            .expect("Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)");
    }

    voice.set_notify_window_message();
    voice.set_interest(winapi::SPFEI(5) | winapi::SPFEI(1) | winapi::SPFEI(2), 0);

    voice.speak_wait("Ready!");

    while let Some(msg) = get_message() {
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
