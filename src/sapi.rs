use winapi;
use ole32;
use user32;

use std::cmp::{min, max};
use std::ptr::null_mut;
use std::mem;
use std::ops::Range;

use window::*;

pub const WM_SAPI_EVENT: winapi::UINT = winapi::WM_APP + 15;

pub struct Com {
    hr: winapi::HRESULT,
}

impl Com {
    pub fn new() -> Com {
        println!("new for Com");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms678543.aspx
        let hr = unsafe { ole32::CoInitialize(null_mut()) };
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
        let mut voice: *mut winapi::ISpVoice = null_mut();
        let mut clsid_spvoice: winapi::CLSID = unsafe { mem::zeroed() };

        unsafe {
            if failed(ole32::CLSIDFromProgID(&"SAPI.SpVoice".to_wide_null()[0],
                                             &mut clsid_spvoice)) {
                panic!("failed for SpVoice at CLSIDFromProgID");
            }

            if failed(ole32::CoCreateInstance(
                &clsid_spvoice,
                null_mut(),
                winapi::CLSCTX_ALL,
                &winapi::UuidOfISpVoice,
                &mut voice as *mut *mut winapi::ISpVoice as *mut *mut winapi::c_void
            )) {
                panic!("failed for SpVoice at CoCreateInstance");
            }
            let mut out = Box::new(SpVoice {
                voice: &mut *voice,
                window: null_mut(),
                edit: null_mut(),
                last_read: Vec::new(),
            });

            let window_class_name = "SAPI_event_window_class_name".to_wide_null();
            user32::RegisterClassW(&winapi::WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc_generic::<SpVoice>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0 as winapi::HINSTANCE,
                hIcon: user32::LoadIconW(0 as winapi::HINSTANCE, winapi::IDI_APPLICATION),
                hCursor: user32::LoadCursorW(0 as winapi::HINSTANCE, winapi::IDI_APPLICATION),
                hbrBackground: 16 as winapi::HBRUSH,
                lpszMenuName: 0 as winapi::LPCWSTR,
                lpszClassName: window_class_name.as_ptr(),
            });
            out.window = user32::CreateWindowExW(0,
                                                 window_class_name.as_ptr(),
                                                 &0u16,
                                                 winapi::WS_OVERLAPPEDWINDOW,
                                                 0,
                                                 0,
                                                 0,
                                                 0,
                                                 user32::GetDesktopWindow(),
                                                 0 as winapi::HMENU,
                                                 0 as winapi::HINSTANCE,
                                                 &mut *out as *mut _ as winapi::LPVOID);

            // https://msdn.microsoft.com/en-us/library/windows/desktop/hh298433.aspx
            out.edit = user32::CreateWindowExW(winapi::WS_EX_CLIENTEDGE,
                                               &"EDIT".to_wide_null()[0],
                                               &0u16,
                                               winapi::WS_CHILD | winapi::WS_VISIBLE |
                                               winapi::WS_VSCROLL |
                                               winapi::WS_BORDER |
                                               winapi::ES_LEFT |
                                               winapi::ES_MULTILINE |
                                               winapi::ES_AUTOVSCROLL |
                                               winapi::ES_NOHIDESEL |
                                               winapi::ES_AUTOVSCROLL,
                                               0,
                                               0,
                                               0,
                                               0,
                                               out.window,
                                               winapi_stub::ID_EDITCHILD,
                                               0 as winapi::HINSTANCE,
                                               null_mut());
            move_window(out.window,
                        &winapi::RECT {
                            left: 0,
                            top: 0,
                            right: 400,
                            bottom: 400,
                        });
            user32::ShowWindow(out.window, winapi::SW_SHOWNORMAL);
            out.set_notify_window_message();
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

    pub fn speak<T: ToWide>(&mut self, string: T) {
        self.last_read = string.to_wide_null();
        set_window_text(self.edit, &self.last_read);
        unsafe { self.voice.Speak(self.last_read.as_ptr(), 19, null_mut()) };
    }

    pub fn wait(&mut self) {
        unsafe { self.voice.WaitUntilDone(winapi::INFINITE) };
    }

    pub fn speak_wait<T: ToWide>(&mut self, string: T) {
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

    fn set_notify_window_message(&mut self) {
        unsafe { self.voice.SetNotifyWindowMessage(self.window, WM_SAPI_EVENT, 0, 0) };
    }

    pub fn set_interest(&mut self, event: u64, queued: u64) {
        unsafe { self.voice.SetInterest(event, queued) };
    }
}

impl<'a> Windowed for SpVoice<'a> {
    fn window_proc(&mut self,
                   msg: winapi::UINT,
                   w_param: winapi::WPARAM,
                   l_param: winapi::LPARAM)
                   -> Option<winapi::LRESULT> {
        match msg {
            winapi::WM_DESTROY => close(),
            winapi::WM_QUERYENDSESSION => close(),
            winapi::WM_ENDSESSION => close(),
            WM_SAPI_EVENT => {
                let window_title = format!("rust_reader saying: {}", self.get_status_word())
                                       .to_wide_null();
                set_console_title(&window_title);
                set_window_text(self.window, &window_title);
                set_edit_selection(self.edit, self.get_status().word_range());
                set_edit_scroll_caret(self.edit);
                return Some(0);
            }
            winapi::WM_CREATE => {}
            winapi::WM_SIZE => {
                let mut rect = get_client_rect(self.window);
                if (w_param <= 2) && rect.right > 0 && rect.bottom > 0 {
                    rect.inset(10);
                    move_window(self.edit, &rect);
                    return Some(0);
                }
            }
            winapi::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param as *mut winapi::MINMAXINFO) };
                data.ptMinTrackSize.x = 180;
                data.ptMinTrackSize.y = 90;
                return Some(0);
            }
            _ => {}
        }
        None
    }
}

impl<'a> Drop for SpVoice<'a> {
    fn drop(&mut self) {
        unsafe { self.voice.Release() };
        println!("drop for SpVoice");
    }
}

pub trait StatusUtil {
    fn word_range(&self) -> Range<usize>;
    fn sent_range(&self) -> Range<usize>;
}

impl StatusUtil for winapi::SPVOICESTATUS {
    fn word_range(&self) -> Range<usize> {
        self.ulInputWordPos as usize..(self.ulInputWordPos + self.ulInputWordLen) as usize
    }
    fn sent_range(&self) -> Range<usize> {
        self.ulInputSentPos as usize..(self.ulInputSentPos + self.ulInputSentLen) as usize
    }
}
