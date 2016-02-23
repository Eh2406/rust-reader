use winapi;
use ole32;
use user32;
use kernel32;

use std::ptr;
use std::mem;
use std::ffi::OsStr;
use std::ops;
use std::os::windows::ffi::OsStrExt;

pub const WM_SAPI_EVENT: u32 = winapi::WM_APP; // the events are WM_APP no matter what we ask for

#[inline]
#[allow(dead_code)]
pub fn failed(hr: winapi::HRESULT) -> bool {
    hr < 0
}

#[inline]
#[allow(dead_code)]
pub fn succeeded(hr: winapi::HRESULT) -> bool {
    !failed(hr)
}

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T: AsRef<OsStr>> ToWide for T {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}

pub struct Com {
    hr: winapi::HRESULT,
}

impl Com {
    pub fn new() -> Com {
        println!("new for Com");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms678543.aspx
        let hr = unsafe { ole32::CoInitialize(ptr::null_mut()) };
        if failed(hr) {
            panic!("failed for Com");
        }
        Com { hr: hr }
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms688715.aspx
        if self.hr != winapi::RPC_E_CHANGED_MODE {
            unsafe {
                ole32::CoUninitialize();
            }
        }
        println!("drop for Com");
    }
}

pub unsafe extern "system" fn window_proc(h_wnd: winapi::HWND,
                                          msg: winapi::UINT,
                                          w_param: winapi::WPARAM,
                                          l_param: winapi::LPARAM)
                                          -> winapi::LRESULT {
    match msg {
        winapi::WM_DESTROY => user32::PostQuitMessage(0),
        winapi::WM_NCCREATE => {
            let data = &mut *(l_param as *mut winapi::CREATESTRUCTW);
            user32::SetWindowLongPtrW(h_wnd,
                                      winapi::GWLP_USERDATA,
                                      data.lpCreateParams as winapi::LONG_PTR);
        }
        WM_SAPI_EVENT => {
            let ptr_voice: winapi::LONG_PTR = user32::GetWindowLongPtrW(h_wnd, winapi::GWLP_USERDATA);
            if ptr_voice > 0 {
                let voice = &mut *(ptr_voice as *mut SpVoice);
                let window_title = format!("rust_reader saying: {}", voice.get_status_word()).to_wide_null();
                kernel32::SetConsoleTitleW(window_title.as_ptr());
            }
        }
        winapi::WM_QUERYENDSESSION => user32::PostQuitMessage(0),
        winapi::WM_ENDSESSION => user32::PostQuitMessage(0),
        _ => {
            // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param)
        }

    }
    return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
}

pub struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602.aspx
    voice: &'a mut winapi::ISpVoice,
    window: winapi::HWND,
    last_read: Vec<u16>,
}

#[allow(dead_code)]
impl<'a> SpVoice<'a> {
    pub fn new() -> Box<SpVoice<'a>> {
        println!("new for SpVoice");
        let mut hr;
        let mut voice: *mut winapi::ISpVoice = ptr::null_mut();
        let sp_voice = "SAPI.SpVoice".to_wide_null();
        let mut clsid_spvoice: winapi::CLSID = unsafe { mem::zeroed() };

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
            let window_class_name = "SAPI_event_window_class_name".to_wide_null();
            user32::RegisterClassW(&winapi::WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0 as winapi::HINSTANCE,
                hIcon: user32::LoadIconW(0 as winapi::HINSTANCE, winapi::winuser::IDI_APPLICATION),
                hCursor: user32::LoadCursorW(0 as winapi::HINSTANCE,
                                             winapi::winuser::IDI_APPLICATION),
                hbrBackground: 16 as winapi::HBRUSH,
                lpszMenuName: 0 as winapi::LPCWSTR,
                lpszClassName: window_class_name.as_ptr(),
            });
            let mut out = Box::new(SpVoice {
                voice: &mut *voice,
                window: ptr::null_mut(),
                last_read: Vec::new(),
            });
            out.window = user32::CreateWindowExW(0,
                                                 window_class_name.as_ptr(),
                                                 &0u16,
                                                 winapi::WS_OVERLAPPEDWINDOW,
                                                 0,
                                                 0,
                                                 400,
                                                 400,
                                                 user32::GetDesktopWindow(),
                                                 0 as winapi::HMENU,
                                                 0 as winapi::HINSTANCE,
                                                 &mut *out as *mut _ as winapi::LPVOID);
            out
        }
    }

    pub fn get_window_handle(&mut self) -> winapi::HWND {
        self.window
    }

    pub fn get_status_word(&mut self) -> String {
        let status = self.get_status();
        String::from_utf16_lossy(&self.last_read[status.word_range()])
    }

    pub fn get_status_sent(&mut self) -> String {
        let status = self.get_status();
        String::from_utf16_lossy(&self.last_read[status.sent_range()])
    }

    pub fn speak(&mut self, string: &str) {
        println!("speaking: {:}", string);
        self.last_read = string.to_wide_null();
        unsafe {
            self.voice.Speak(self.last_read.as_ptr(), 19, ptr::null_mut());
        }
    }

    pub fn wait(&mut self) {
        unsafe {
            self.voice.WaitUntilDone(winapi::INFINITE);
        }
    }

    pub fn speak_wait(&mut self, string: &str) {
        self.speak(string);
        self.wait();
    }

    pub fn pause(&mut self) {
        unsafe {
            self.voice.Pause();
        }
    }

    pub fn resume(&mut self) {
        unsafe {
            self.voice.Resume();
        }
    }

    pub fn set_rate(&mut self, rate: i32) {
        unsafe {
            self.voice.SetRate(rate);
        }
    }

    pub fn get_rate(&mut self) -> i32 {
        let mut rate = 0;
        unsafe {
            self.voice.GetRate(&mut rate);
        }
        rate
    }

    pub fn set_volume(&mut self, volume: u16) {
        unsafe {
            self.voice.SetVolume(volume);
        }
    }

    pub fn get_volume(&mut self) -> u16 {
        let mut volume = 0;
        unsafe {
            self.voice.GetVolume(&mut volume);
        }
        volume
    }

    pub fn set_alert_boundary(&mut self, boundary: winapi::SPEVENTENUM) {
        unsafe {
            self.voice.SetAlertBoundary(boundary);
        }
    }

    pub fn get_alert_boundary(&mut self) -> winapi::SPEVENTENUM {
        let mut boundary = winapi::SPEVENTENUM(0);
        unsafe {
            self.voice.GetAlertBoundary(&mut boundary);
        }
        boundary
    }

    pub fn get_status(&mut self) -> winapi::SPVOICESTATUS {
        let mut status: winapi::SPVOICESTATUS = unsafe { mem::zeroed() };
        unsafe {
            self.voice.GetStatus(&mut status, 0u16 as *mut *mut u16);
        }
        status
    }

    pub fn set_notify_window_message(&mut self) {
        // the events are WM_APP no matter what we ask for
        unsafe {
            self.voice.SetNotifyWindowMessage(self.window, WM_SAPI_EVENT, 0, 0);
        }
    }

    pub fn set_interest(&mut self, event: u64, queued: u64) {
        unsafe {
            self.voice.SetInterest(event, queued);
        }
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

pub trait StatusUtil {
    fn word_range(&self) -> ops::Range<usize>;
    fn sent_range(&self) -> ops::Range<usize>;
}

impl StatusUtil for winapi::SPVOICESTATUS {
    fn word_range(&self) -> ops::Range<usize> {
        self.ulInputWordPos as usize..(self.ulInputWordPos + self.ulInputWordLen) as usize
    }
    fn sent_range(&self) -> ops::Range<usize> {
        self.ulInputSentPos as usize..(self.ulInputSentPos + self.ulInputSentLen) as usize
    }
}
