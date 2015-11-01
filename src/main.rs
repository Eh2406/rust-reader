extern crate winapi;
extern crate ole32;
extern crate user32;
extern crate clipboard_win;

use std::ptr;

mod sapi;
use sapi::*;

mod clipboard;
use clipboard::*;

fn main() {
    let _com = Com::new();
    let mut voice = SpVoice::new();
    voice.set_volume(99);
    println!("volume :{:?}", voice.get_volume());
    voice.set_rate(6);
    println!("rate :{:?}", voice.get_rate());
    voice.speak_wait("Ready!");
    unsafe {
        user32::RegisterHotKey(ptr::null_mut(), 0, 2, 191); // ctrl-? key
        user32::RegisterHotKey(ptr::null_mut(), 1, 7, winapi::VK_ESCAPE as u32); // ctrl-alt-shift-esk
        user32::RegisterHotKey(ptr::null_mut(), 2, 7, 191); // ctrl-alt-shift-?
        user32::RegisterHotKey(ptr::null_mut(), 3, 2, winapi::VK_OEM_PERIOD as u32); // ctrl-.
    }
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
    unsafe {
        user32::UnregisterHotKey(ptr::null_mut(), 0);
        user32::UnregisterHotKey(ptr::null_mut(), 1);
        user32::UnregisterHotKey(ptr::null_mut(), 2);
        user32::UnregisterHotKey(ptr::null_mut(), 3);
    }
    voice.resume();
    voice.speak_wait("bye!");
}
