use clipboard_win::{get_clipboard_string, set_clipboard_string};
use error_code::ErrorCode;
use std::mem;
use std::num::NonZeroU32;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse;

fn get_clipboard_seq_num() -> Option<NonZeroU32> {
    clipboard_win::seq_num()
}

pub fn clipboard_setup() {
    if get_clipboard_seq_num().is_none() {
        // this will crash on our reading so lets get it over with.
        // this may fix the problem
        set_clipboard_string("").unwrap();
        // let us see if it did
        get_clipboard_seq_num()
            .expect("Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)");
    }
}

pub fn send_key_event(
    vk: KeyboardAndMouse::VIRTUAL_KEY,
    flags: KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) {
    let mut input: KeyboardAndMouse::INPUT = unsafe { mem::zeroed() };
    unsafe {
        input.r#type = KeyboardAndMouse::INPUT_KEYBOARD;
        input.Anonymous.ki = KeyboardAndMouse::KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        KeyboardAndMouse::SendInput(&[input], mem::size_of::<KeyboardAndMouse::INPUT>() as i32);
    }
}

pub fn press_key(vk: &[u16]) {
    for &v in vk {
        send_key_event(
            KeyboardAndMouse::VIRTUAL_KEY(v),
            KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        );
    }
    sleep(Duration::from_millis(1));
    for &v in vk.iter().rev() {
        send_key_event(
            KeyboardAndMouse::VIRTUAL_KEY(v),
            KeyboardAndMouse::KEYEVENTF_KEYUP,
        );
    }
}

pub fn press_ctrl_c() {
    println!("sending ctrl-c");
    press_key(&[KeyboardAndMouse::VK_CONTROL.0, 67]); //ascii for "c"
}

pub fn what_on_clipboard_seq_num(clip_num: NonZeroU32, n: u8) -> bool {
    for i in 0..u32::from(n) {
        if get_clipboard_seq_num().unwrap_or(clip_num) != clip_num {
            return true;
        }
        sleep(Duration::from_millis(2u64.pow(i)));
    }
    get_clipboard_seq_num().unwrap_or(clip_num) != clip_num
}

pub fn what_on_get_clipboard_string(n: u8) -> clipboard_win::SysResult<String> {
    for i in 0..u32::from(n) {
        match get_clipboard_string() {
            Ok(x) => return Ok(x),
            Err(_) => sleep(Duration::from_millis(2u64.pow(i))),
        }
    }
    get_clipboard_string()
}

pub fn get_text() -> clipboard_win::SysResult<String> {
    println!("getting text");
    let old_clip = what_on_get_clipboard_string(6);
    let old_clip_num = get_clipboard_seq_num()
        .expect("Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)");
    press_ctrl_c();
    if !what_on_clipboard_seq_num(old_clip_num, 6) {
        return Err(ErrorCode::new(0));
    }
    let new_clip = what_on_get_clipboard_string(6);
    if let Ok(clip) = old_clip {
        let _ = set_clipboard_string(&clip);
    }
    new_clip
}
