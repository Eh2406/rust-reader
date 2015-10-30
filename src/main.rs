extern crate winapi;
extern crate ole32;
extern crate user32;
extern crate clipboard_win;

use clipboard_win::{get_clipboard_string, set_clipboard};
use clipboard_win::wrapper::get_clipboard_seq_num;
use std::mem;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::fmt::Display;

#[inline]
#[allow(dead_code)]
fn failed(hr: winapi::HRESULT) -> bool {
    hr < 0
}

#[inline]
#[allow(dead_code)]
fn succeeded(hr: winapi::HRESULT) -> bool {
    !failed(hr)
}

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T> ToWide for T where T: AsRef<OsStr> {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}

struct Com {
    hr: winapi::HRESULT,
}

impl Com {
    fn new() -> Com {
        println!("new for Com");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms678543(v=vs.85).aspx
        let hr = unsafe {ole32::CoInitialize(ptr::null_mut())};
        if failed(hr) {
            panic!("failed for Com");
        }
        Com {hr: hr}
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms688715(v=vs.85).aspx
        if self.hr != winapi::RPC_E_CHANGED_MODE {
            unsafe {
                ole32::CoUninitialize();
            }
        }
        println!("drop for Com");
    }
}

struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602(VS.85).aspx
    voice: &'a mut winapi::ISpVoice,
}

#[allow(dead_code)]
impl<'a> SpVoice<'a> {
    fn new() -> SpVoice<'a> {
        println!("new for SpVoice");
        let mut hr;
        let mut voice: *mut winapi::ISpVoice = ptr::null_mut();
        let sp_voice = "SAPI.SpVoice".to_wide_null();
        let mut clsid_spvoice = winapi::CLSID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        };

        unsafe {
            hr = ole32::CLSIDFromProgID(&sp_voice[0], &mut clsid_spvoice);
            if failed(hr) {
                panic!("failed for SpVoice at CLSIDFromProgID");
            }

            hr = ole32::CoCreateInstance(
                &clsid_spvoice,
                ptr::null_mut(),
                winapi::CLSCTX_ALL,
                &winapi::UuidOfISpVoice,
                &mut voice as *mut *mut winapi::ISpVoice as *mut *mut winapi::c_void
            );
            if failed(hr) {
                panic!("failed for SpVoice at CoCreateInstance");
            }
            SpVoice {
                voice: &mut *voice,
            }
        }
    }

    fn speak<T: ToWide + Display> (&mut self, string: T) {
        unsafe {
            println!("speaking: {:}", string);
            self.voice.Speak(string.to_wide_null().as_ptr(), 19, ptr::null_mut());
        }
    }

    fn wait (&mut self) {
        unsafe {
            self.voice.WaitUntilDone(winapi::INFINITE);
        }
    }

    fn speak_wait<T: ToWide + Display> (&mut self, string: T) {
        self.speak(string);
        self.wait();
    }

    fn pause (&mut self) {
        unsafe {
            self.voice.Pause();
        }
    }

    fn resume (&mut self) {
        unsafe {
            self.voice.Resume();
        }
    }

    fn set_rate (&mut self, rate: i32) {
        unsafe {
            self.voice.SetRate(rate);
        }
    }

    fn get_rate (&mut self) -> i32 {
        let mut rate = 0;
        unsafe {
            self.voice.GetRate(&mut rate);
        }
        rate
    }

    fn set_volume (&mut self, volume: u16) {
        unsafe {
            self.voice.SetVolume(volume);
        }
    }

    fn get_volume (&mut self) -> u16 {
        let mut volume = 0;
        unsafe {
            self.voice.GetVolume(&mut volume);
        }
        volume
    }

    fn get_status (&mut self) -> winapi::SPVOICESTATUS {
        let mut status = winapi::SPVOICESTATUS {
                                        ulCurrentStream: 0,
                                        ulLastStreamQueued: 0,
                                        hrLastResult: 0,
                                        dwRunningState: 0,
                                        ulInputWordPos: 0,
                                        ulInputWordLen: 0,
                                        ulInputSentPos: 0,
                                        ulInputSentLen: 0,
                                        lBookmarkId: 0,
                                        PhonemeId: 0,
                                        VisemeId: winapi::SP_VISEME_0,
                                        dwReserved1: 0,
                                        dwReserved2: 0,
        };
        unsafe {
            self.voice.GetStatus(&mut status, 0u16 as *mut *mut u16);
        }
        status
    }
}

impl<'a> Drop for SpVoice<'a> {
    fn drop(&mut self) {
        unsafe {
            self.voice.Release();
        }
        println!("drop for SpVoice");
    }
}

fn send_key_event(vk: u16, flags: u32) {
    let mut input = winapi::INPUT {
        type_: winapi::INPUT_KEYBOARD,
        u: [0u32; 6]
    };
    unsafe {
        *input.ki_mut() = winapi::KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,};
        let mut b = &mut input;
        user32::SendInput(1, b, mem::size_of::<winapi::INPUT>() as i32);
    }
}

fn send_ctrl_c() {
    use winapi::{VK_CONTROL, KEYEVENTF_KEYUP};
    println!("sending ctrl-c");
    send_key_event(VK_CONTROL as u16, 0);
    send_key_event(67, 0); //ascii for "c"
    send_key_event(67, KEYEVENTF_KEYUP); //ascii for "c"
    send_key_event(VK_CONTROL as u16, KEYEVENTF_KEYUP);
}

fn what_on_clipboard_seq_num(clip_num: u32, n: u32) -> bool {
    for i in 1..(n + 1) {
        if get_clipboard_seq_num().unwrap_or(clip_num) != clip_num {
            return true;
        }
        std::thread::sleep_ms(10 * i);
    }
    get_clipboard_seq_num().unwrap_or(clip_num) != clip_num
}

fn get_text() -> Result<String, clipboard_win::WindowsError> {
    println!("geting text");
    let old_clip = get_clipboard_string();
    let old_clip_num = get_clipboard_seq_num().unwrap_or_else(|| panic!("Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)"));
    send_ctrl_c();
    if !what_on_clipboard_seq_num(old_clip_num, 15) {
        return Err(clipboard_win::WindowsError::new(0));
    }
    let new_clip = get_clipboard_string();
    if let Ok(clip) = old_clip {
        let _ = set_clipboard(&clip);
    }
    new_clip
}

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
