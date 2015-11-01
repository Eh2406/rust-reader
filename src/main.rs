extern crate winapi;
extern crate ole32;
extern crate user32;
extern crate clipboard_win;

use std::ptr;

mod sapi;
use sapi::*;

mod clipboard;
use clipboard::*;

mod hot_key;
use hot_key::*;

fn main() {
    let _com = Com::new();
    let mut voice = SpVoice::new();
    voice.set_volume(99);
    println!("volume :{:?}", voice.get_volume());
    voice.set_rate(6);
    println!("rate :{:?}", voice.get_rate());
    voice.speak_wait("Ready!");
    let _hk = [ // TODO why do we nead to spesify the id.
        HotKey::new(2, 191, 0), // ctrl-? key
        HotKey::new(7, winapi::VK_ESCAPE as u32, 1), // ctrl-alt-shift-esk
        HotKey::new(7, 191, 2), // ctrl-alt-shift-?
        HotKey::new(2, winapi::VK_OEM_PERIOD as u32, 3), // ctrl-.
    ];
    let mut msg = winapi::MSG {
        hwnd: ptr::null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: winapi::POINT {
            x: 0,
            y: 0,
        },
    };
    while unsafe {user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0)} > 0 {
        match msg.message {
            winapi::WM_HOTKEY => {
                match msg.wParam {
                    0 => {
                        voice.resume();
                        match get_text() {
                            Ok(x) => voice.speak(x),
                            Err(x) => {
                                voice.speak_wait("oops. error.");
                                println!("{:?}", x);
                            }
                        }
                    }
                    1 => {
                        break;
                    }
                    2 => {
                        println!("dwRunningState {}", voice.get_status().dwRunningState)
                    }
                    3 => {
                        match voice.get_status().dwRunningState {
                            2 => voice.pause(),
                            _ => voice.resume(),
                        }
                    }
                    _ => {
                        println!("unknown hot {}", msg.wParam);
                    }
                }
            }
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
