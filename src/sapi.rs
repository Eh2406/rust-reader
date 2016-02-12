use winapi;
use ole32;
use user32;

use std::ptr;
use std::mem;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::fmt::Display;

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

pub struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602.aspx
    voice: &'a mut winapi::ISpVoice,
    window: winapi::HWND,
}

#[allow(dead_code)]
impl<'a> SpVoice<'a> {
    pub fn new() -> SpVoice<'a> {
        println!("new for SpVoice");
        let mut hr;
        let sapi_event_window;
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
                lpfnWndProc: Some(user32::DefWindowProcW),
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
            sapi_event_window = user32::CreateWindowExW(0,
                                                        window_class_name.as_ptr(),
                                                        &0u16,
                                                        winapi::WS_OVERLAPPEDWINDOW,
                                                        0,
                                                        0,
                                                        400,
                                                        400,
                                                        winapi::HWND_MESSAGE,
                                                        0 as winapi::HMENU,
                                                        0 as winapi::HINSTANCE,
                                                        ptr::null_mut());
            SpVoice {
                voice: &mut *voice,
                window: sapi_event_window,
            }
        }
    }

    pub fn get_window_handle(&mut self) -> winapi::HWND {
        self.window
    }

    pub fn speak<T: ToWide + Display>(&mut self, string: T) {
        unsafe {
            println!("speaking: {:}", string);
            self.voice.Speak(string.to_wide_null().as_ptr(), 19, ptr::null_mut());
        }
    }

    pub fn wait(&mut self) {
        unsafe {
            self.voice.WaitUntilDone(winapi::INFINITE);
        }
    }

    pub fn speak_wait<T: ToWide + Display>(&mut self, string: T) {
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
