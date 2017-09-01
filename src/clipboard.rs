use winapi;
use user32;
use clipboard_win::{get_clipboard_string, set_clipboard_string, Clipboard};
use std::mem;
use std::thread::sleep;
use std::time::Duration;
use std::io;

fn get_clipboard_seq_num() -> Option<u32> {
    Clipboard::seq_num()
}

pub fn clipboard_setup() {
    if get_clipboard_seq_num().is_none() {
        // this will crash on our reading so lets get it over with.
        // this may fix the problem
        set_clipboard_string("").unwrap();
        // let us see if it did
        get_clipboard_seq_num().expect(
            "Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)",
        );
    }
}

pub trait NewINPUT {
    fn new() -> winapi::INPUT;
}

#[cfg(target_arch = "x86")]
impl NewINPUT for winapi::INPUT {
    fn new() -> winapi::INPUT {
        winapi::INPUT {
            type_: winapi::INPUT_KEYBOARD,
            u: [0u32; 6],
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl NewINPUT for winapi::INPUT {
    fn new() -> winapi::INPUT {
        winapi::INPUT {
            type_: winapi::INPUT_KEYBOARD,
            u: [0u64; 4],
        }
    }
}

pub fn send_key_event(vk: u16, flags: u32) {
    let mut input = winapi::INPUT::new();
    unsafe {
        *input.ki_mut() = winapi::KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        let mut b = &mut input;
        user32::SendInput(1, b, mem::size_of::<winapi::INPUT>() as i32);
    }
}

pub fn press_key(vk: &[u16]) {
    for &v in vk {
        send_key_event(v, 0);
    }
    sleep(Duration::from_millis(1));
    for &v in vk.iter().rev() {
        send_key_event(v, winapi::KEYEVENTF_KEYUP);
    }
}

pub fn press_ctrl_c() {
    println!("sending ctrl-c");
    press_key(&[winapi::VK_CONTROL as u16, 67]); //ascii for "c"
}

pub fn what_on_clipboard_seq_num(clip_num: u32, n: u8) -> bool {
    for i in 0..(n as u32) {
        if get_clipboard_seq_num().unwrap_or(clip_num) != clip_num {
            return true;
        }
        sleep(Duration::from_millis((2 as u64).pow(i)));
    }
    get_clipboard_seq_num().unwrap_or(clip_num) != clip_num
}

pub fn what_on_get_clipboard_string(n: u8) -> io::Result<String> {
    for i in 0..(n as u32) {
        match get_clipboard_string() {
            Ok(x) => return Ok(x),
            Err(_) => sleep(Duration::from_millis((2 as u64).pow(i))),
        }
    }
    get_clipboard_string()
}

pub fn get_text() -> io::Result<String> {
    println!("getting text");
    let old_clip = what_on_get_clipboard_string(6);
    let old_clip_num = get_clipboard_seq_num().expect(
        "Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)",
    );
    press_ctrl_c();
    if !what_on_clipboard_seq_num(old_clip_num, 6) {
        return Err(io::Error::new(io::ErrorKind::Other, "oh no!"));
    }
    let new_clip = what_on_get_clipboard_string(6);
    if let Ok(clip) = old_clip {
        let _ = set_clipboard_string(&clip);
    }
    new_clip
}
