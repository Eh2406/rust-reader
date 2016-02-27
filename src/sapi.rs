use winapi;
use ole32;
use user32;
use kernel32;

use std::cmp::{min, max};
use std::ptr;
use std::mem;
use std::ffi::OsStr;
use std::ops;
use std::os::windows::ffi::OsStrExt;

pub const WM_SAPI_EVENT: u32 = winapi::WM_APP + 15;

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
            unsafe { ole32::CoUninitialize() };
        }
        println!("drop for Com");
    }
}

pub fn get_window_wrapper<'a, T>(h_wnd: winapi::HWND) -> Option<&'a mut T> {
    let ptr: winapi::LONG_PTR = unsafe { user32::GetWindowLongPtrW(h_wnd, winapi::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

pub fn set_window_wrapper(h_wnd: winapi::HWND, l_param: winapi::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winapi::CREATESTRUCTW) };
    unsafe {
        user32::SetWindowLongPtrW(h_wnd,
                                  winapi::GWLP_USERDATA,
                                  data.lpCreateParams as winapi::LONG_PTR);
    }
}

pub fn set_console_title(title: &Vec<u16>) -> i32 {
    unsafe { kernel32::SetConsoleTitleW(title.as_ptr()) }
}

pub fn set_window_text(h_wnd: winapi::HWND, wide: &Vec<u16>) -> winapi::BOOL {
    unsafe { user32::SetWindowTextW(h_wnd, wide.as_ptr()) }
}

pub fn close() {
    unsafe { user32::PostQuitMessage(0) }
}

pub fn set_edit_selection(h_wnd: winapi::HWND, celec: ops::Range<usize>) -> winapi::LRESULT {
    // EM_SETSEL
    unsafe {
        user32::SendMessageW(h_wnd,
                             177,
                             celec.start as winapi::WPARAM,
                             celec.end as winapi::LPARAM)
    }
}

pub fn get_client_rect(h_wnd: winapi::HWND) -> winapi::RECT {
    let mut rec: winapi::RECT = unsafe { mem::zeroed() };
    unsafe { user32::GetClientRect(h_wnd, &mut rec) };
    rec
}

pub unsafe extern "system" fn window_proc(h_wnd: winapi::HWND,
                                          msg: winapi::UINT,
                                          w_param: winapi::WPARAM,
                                          l_param: winapi::LPARAM)
                                          -> winapi::LRESULT {
    match msg {
        winapi::WM_DESTROY => close(),
        winapi::WM_QUERYENDSESSION => close(),
        winapi::WM_ENDSESSION => close(),
        winapi::WM_NCCREATE => set_window_wrapper(h_wnd, l_param),
        WM_SAPI_EVENT => {
            if let Some(voice) = get_window_wrapper::<SpVoice>(h_wnd) {
                let window_title = format!("rust_reader saying: {}", voice.get_status_word())
                                       .to_wide_null();
                set_console_title(&window_title);
                set_window_text(voice.window, &window_title);
                set_edit_selection(voice.edit, voice.get_status().word_range());
                user32::SendMessageW(voice.edit,
                                     183, // EM_SCROLLCARET
                                     0 as winapi::WPARAM,
                                     0 as winapi::LPARAM);
                return 0;
            }
        }
        winapi::WM_SIZE => {
            if let Some(voice) = get_window_wrapper::<SpVoice>(h_wnd) {
                let rect = get_client_rect(voice.window);
                if (w_param <= 2) && rect.right > 0 && rect.bottom > 0 {
                    user32::MoveWindow(voice.edit,
                                       10,
                                       10,
                                       rect.right - 10 - 13,
                                       rect.bottom - 10 - 10,
                                       winapi::TRUE);
                    return 0;
                }
            }
        }
        /* next winapi bump
        winapi::WM_GETMINMAXINFO => {
            let data = unsafe { &mut *(l_param as *mut winapi::MINMAXINFO) };
            data.ptMinTrackSize.x = 160;
            data.ptMinTrackSize.y = 90;
            return 0;
        }
        */
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
    edit: winapi::HWND,
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
            let mut out = Box::new(SpVoice {
                voice: &mut *voice,
                window: ptr::null_mut(),
                edit: ptr::null_mut(),
                last_read: Vec::new(),
            });

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
            out.window = user32::CreateWindowExW(0,
                                                 window_class_name.as_ptr(),
                                                 &0u16,
                                                 winapi::WS_OVERLAPPEDWINDOW | winapi::WS_VISIBLE,
                                                 0,
                                                 0,
                                                 400,
                                                 400,
                                                 user32::GetDesktopWindow(),
                                                 0 as winapi::HMENU,
                                                 0 as winapi::HINSTANCE,
                                                 &mut *out as *mut _ as winapi::LPVOID);

            // https://msdn.microsoft.com/en-us/library/windows/desktop/hh298433.aspx
            let window_class_name = "EDIT".to_wide_null();
            out.edit = user32::CreateWindowExW(winapi::WS_EX_CLIENTEDGE,
                                               window_class_name.as_ptr(),
                                               &0u16,
                                               winapi::WS_CHILD | winapi::WS_VISIBLE |
                                               winapi::WS_VSCROLL |
                                               winapi::WS_BORDER |
                                               0 |
                                               4 |
                                               64 |
                                               256 |
                                               64,
                                               // | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_NOHIDESEL | ES_AUTOVSCROLL
                                               // http://www.math.uiuc.edu/~gfrancis/illimath/windows/aszgard_mini/bin/MinGW/include/winuser.h
                                               10,
                                               10,
                                               367,
                                               340,
                                               out.window,
                                               100 as winapi::HMENU, // winapi::ID_EDITCHILD
                                               0 as winapi::HINSTANCE,
                                               ptr::null_mut());

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
        self.last_read = string.to_wide_null();
        set_window_text(self.edit, &self.last_read);
        unsafe { self.voice.Speak(self.last_read.as_ptr(), 19, ptr::null_mut()) };
    }

    pub fn wait(&mut self) {
        unsafe { self.voice.WaitUntilDone(winapi::INFINITE) };
    }

    pub fn speak_wait(&mut self, string: &str) {
        self.speak(string);
        self.wait();
    }

    pub fn pause(&mut self) {
        unsafe { self.voice.Pause() };
    }

    pub fn resume(&mut self) {
        unsafe { self.voice.Resume() };
    }

    pub fn set_rate(&mut self, rate: i32) -> i32 {
        let rate = max(min(rate, 10), -10);
        unsafe { self.voice.SetRate(rate) };
        rate
    }

    pub fn get_rate(&mut self) -> i32 {
        let mut rate = 0;
        unsafe { self.voice.GetRate(&mut rate) };
        rate
    }

    pub fn change_rate(&mut self, delta: i32) -> i32 {
        let rate = self.get_rate() + delta;
        self.set_rate(rate)
    }

    pub fn set_volume(&mut self, volume: u16) {
        unsafe { self.voice.SetVolume(min(volume, 100)) };
    }

    pub fn get_volume(&mut self) -> u16 {
        let mut volume = 0;
        unsafe { self.voice.GetVolume(&mut volume) };
        volume
    }

    pub fn set_alert_boundary(&mut self, boundary: winapi::SPEVENTENUM) {
        unsafe { self.voice.SetAlertBoundary(boundary) };
    }

    pub fn get_alert_boundary(&mut self) -> winapi::SPEVENTENUM {
        let mut boundary = winapi::SPEVENTENUM(0);
        unsafe { self.voice.GetAlertBoundary(&mut boundary) };
        boundary
    }

    pub fn get_status(&mut self) -> winapi::SPVOICESTATUS {
        let mut status: winapi::SPVOICESTATUS = unsafe { mem::zeroed() };
        unsafe { self.voice.GetStatus(&mut status, 0u16 as *mut *mut u16) };
        status
    }

    pub fn set_notify_window_message(&mut self) {
        unsafe { self.voice.SetNotifyWindowMessage(self.window, WM_SAPI_EVENT, 0, 0) };
    }

    pub fn set_interest(&mut self, event: u64, queued: u64) {
        unsafe { self.voice.SetInterest(event, queued) };
    }
}

impl<'a> Drop for SpVoice<'a> {
    fn drop(&mut self) {
        unsafe { self.voice.Release() };
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
