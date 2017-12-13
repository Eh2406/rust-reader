use winapi;
use ole32;
use user32;
use chrono;

use std::cmp::{min, max};
use std::mem;
use std::ptr::null_mut;
use std::time::Instant;
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

const SETTING_BUTTON: winapi::WPARAM = 101;

#[derive(Debug)]
pub struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602.aspx
    voice: &'a mut winapi::ISpVoice,
    window: winapi::HWND,
    edit: winapi::HWND,
    rate: winapi::HWND,
    reload_settings: winapi::HWND,
    last_read: WideString,
    last_update: Option<(Instant, Range<usize>)>,
    us_per_utf16: i64,
}

impl<'a> SpVoice<'a> {
    pub fn new<'c: 'a>(_con: &'c Com) -> Box<SpVoice<'a>> {
        println!("new for SpVoice");
        let mut voice: *mut winapi::ISpVoice = null_mut();
        let mut clsid_spvoice: winapi::CLSID = unsafe { mem::zeroed() };
        let sapi_id: WideString = "SAPI.SpVoice".into();

        unsafe {
            if failed(ole32::CLSIDFromProgID(sapi_id.as_ptr(), &mut clsid_spvoice)) {
                panic!("failed for SpVoice at CLSIDFromProgID");
            }

            if failed(ole32::CoCreateInstance(
                &clsid_spvoice,
                null_mut(),
                winapi::CLSCTX_ALL,
                &winapi::UuidOfISpVoice,
                &mut voice as *mut *mut winapi::ISpVoice as *mut *mut winapi::c_void,
            ))
            {
                panic!("failed for SpVoice at CoCreateInstance");
            }
            let mut out = Box::new(SpVoice {
                voice: &mut *voice,
                window: null_mut(),
                edit: null_mut(),
                rate: null_mut(),
                reload_settings: null_mut(),
                last_read: WideString::new(),
                last_update: None,
                us_per_utf16: 20_000,
            });

            let window_class_name: WideString = "SAPI_event_window_class_name".into();
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
            out.window = user32::CreateWindowExW(
                0,
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
                &mut *out as *mut _ as winapi::LPVOID,
            );

            // https://msdn.microsoft.com/en-us/library/windows/desktop/hh298433.aspx
            let wide_edit: WideString = "EDIT".into();
            out.edit = user32::CreateWindowExW(
                winapi::WS_EX_CLIENTEDGE,
                wide_edit.as_ptr(),
                &0u16,
                winapi::WS_CHILD | winapi::WS_VISIBLE | winapi::WS_VSCROLL |
                    winapi::WS_BORDER | winapi::ES_LEFT | winapi::ES_MULTILINE |
                    winapi::ES_AUTOVSCROLL |
                    winapi::ES_NOHIDESEL | winapi::ES_AUTOVSCROLL,
                0,
                0,
                0,
                0,
                out.window,
                winapi_stub::ID_EDITCHILD,
                0 as winapi::HINSTANCE,
                null_mut(),
            );
            let wide_static: WideString = "STATIC".into();
            out.rate = user32::CreateWindowExW(
                0,
                wide_static.as_ptr(),
                &0u16,
                winapi::WS_CHILD | winapi::WS_VISIBLE | winapi_stub::SS_CENTER |
                    winapi_stub::SS_NOPREFIX,
                0,
                0,
                0,
                0,
                out.window,
                0 as winapi::HMENU,
                0 as winapi::HINSTANCE,
                null_mut(),
            );
            let wide_button: WideString = "BUTTON".into();
            let wide_settings: WideString = "Reload Settings".into();
            out.reload_settings = user32::CreateWindowExW(
                0,
                wide_button.as_ptr(),
                wide_settings.as_ptr(),
                winapi::WS_TABSTOP | winapi::WS_VISIBLE | winapi::WS_CHILD |
                    winapi::BS_DEFPUSHBUTTON,
                10,
                10,
                20,
                20,
                out.window,
                SETTING_BUTTON as winapi::HMENU,
                0 as winapi::HINSTANCE,
                null_mut(),
            );
            move_window(
                out.window,
                &winapi::RECT {
                    left: 0,
                    top: 0,
                    right: 400,
                    bottom: 400,
                },
            );
            show_window(out.window, winapi::SW_SHOWNORMAL);
            out.set_notify_window_message();
            out.set_volume(100);
            out.set_alert_boundary(winapi::SPEI_PHONEME);
            out.set_interest(&[5, 1, 2], &[]);
            out
        }
    }

    #[allow(dead_code)]
    pub fn get_window_handle(&mut self) -> winapi::HWND {
        self.window
    }

    pub fn toggle_window_visible(&self) -> winapi::BOOL {
        toggle_window_visible(self.window)
    }

    #[allow(dead_code)]
    pub fn get_status_word(&mut self) -> String {
        let status = self.get_status();
        self.last_read.get_slice(status.word_range())
    }

    #[allow(dead_code)]
    pub fn get_status_sent(&mut self) -> String {
        let status = self.get_status();
        self.last_read.get_slice(status.sent_range())
    }

    pub fn speak<T: Into<WideString>>(&mut self, string: T) {
        self.last_read = string.into();
        set_window_text(self.edit, &self.last_read);
        unsafe { self.voice.Speak(self.last_read.as_ptr(), 19, null_mut()) };
        self.last_update = None;
    }

    pub fn wait(&mut self) {
        unsafe { self.voice.WaitUntilDone(winapi::INFINITE) };
    }

    pub fn speak_wait<T: Into<WideString>>(&mut self, string: T) {
        self.speak(string);
        self.wait();
    }

    pub fn pause(&mut self) {
        unsafe { self.voice.Pause() };
        self.last_update = None;
    }

    pub fn resume(&mut self) {
        unsafe { self.voice.Resume() };
        self.last_update = None;
    }

    pub fn set_rate(&mut self, rate: i32) -> i32 {
        let rate = max(min(rate, 10), -10);
        unsafe { self.voice.SetRate(rate) };
        self.last_update = None;
        self.get_rate()
    }

    pub fn get_rate(&mut self) -> i32 {
        let mut rate = 0;
        unsafe { self.voice.GetRate(&mut rate) };
        set_window_text(self.rate, &format!("reading at rate: {}", rate).into());
        rate
    }

    pub fn change_rate(&mut self, delta: i32) -> i32 {
        let rate = self.get_rate() + delta;
        self.set_rate(rate)
    }

    pub fn set_volume(&mut self, volume: u16) {
        unsafe { self.voice.SetVolume(min(volume, 100)) };
    }

    #[allow(dead_code)]
    pub fn get_volume(&mut self) -> u16 {
        let mut volume = 0;
        unsafe { self.voice.GetVolume(&mut volume) };
        volume
    }

    pub fn set_alert_boundary(&mut self, boundary: winapi::SPEVENTENUM) {
        unsafe { self.voice.SetAlertBoundary(boundary) };
    }

    #[allow(dead_code)]
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
        unsafe {
            self.voice.SetNotifyWindowMessage(
                self.window,
                WM_SAPI_EVENT,
                0,
                0,
            )
        };
    }

    pub fn set_interest(&mut self, event: &[u64], queued: &[u64]) {
        let queued = queued.iter().map(|&x| winapi::SPFEI(x)).fold(
            0u64,
            |acc, x| acc | x,
        );
        let event = event.iter().map(|&x| winapi::SPFEI(x)).fold(
            queued,
            |acc, x| acc | x,
        );
        unsafe { self.voice.SetInterest(event, queued) };
    }
}

impl<'a> Windowed for SpVoice<'a> {
    fn window_proc(
        &mut self,
        msg: winapi::UINT,
        w_param: winapi::WPARAM,
        l_param: winapi::LPARAM,
    ) -> Option<winapi::LRESULT> {
        match msg {
            winapi::WM_DESTROY |
            winapi::WM_QUERYENDSESSION |
            winapi::WM_ENDSESSION => close(),
            WM_SAPI_EVENT => {
                let word_range = self.get_status().word_range();
                if let Some((ref old_time, ref old_word_range)) = self.last_update {
                    if old_word_range.start == word_range.start {
                        return Some(0);
                    }
                    let elapsed = chrono::Duration::from_std(old_time.elapsed()).expect("bad time diffrence.").num_microseconds().expect("bad time diffrence.");
                    let new_rate = elapsed / ((word_range.start - old_word_range.start) as i64);
                    self.us_per_utf16 = (self.us_per_utf16 * 1000 + new_rate) / 1001;
                }
                self.last_update = Some((Instant::now(), word_range.clone()));
                let ms_left = chrono::Duration::microseconds(((self.last_read.len() - word_range.end) as i64) * self.us_per_utf16);
                let window_title = format!(
                    "{:.1}% {}s \"{}\" rust_reader",
                    100.0 * ((word_range.end + 2) as f64) / (self.last_read.len() as f64),
                    ms_left.num_seconds(),
                    self.last_read.get_slice(word_range.clone())
                ).into();
                set_console_title(&window_title);
                set_window_text(self.window, &window_title);
                set_edit_selection(self.edit, &word_range);
                set_edit_scroll_caret(self.edit);
                return Some(0);
            }
            winapi::WM_SIZE => {
                let mut rect = get_client_rect(self.window);
                if (w_param <= 2) && rect.right > 0 && rect.bottom > 0 {
                    rect.inset(3);
                    let (up, mut down) = rect.split_rows(25);
                    down.inset(3);
                    move_window(self.edit, &down);
                    let (mut left, mut right) = up.split_columns(120);
                    left.inset(3);
                    right.inset(3);
                    move_window(self.reload_settings, &left);
                    move_window(self.rate, &right);
                    self.get_rate(); //force repaint of text
                    return Some(0);
                }
            }
            winapi::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param as *mut winapi::MINMAXINFO) };
                data.ptMinTrackSize.x = 300;
                data.ptMinTrackSize.y = 110;
                return Some(0);
            }
            winapi::WM_COMMAND => {
                use press_hotkey;
                use Action;
                match w_param {
                    SETTING_BUTTON => {
                        press_hotkey(Action::ReloadSettings);
                        return Some(0);
                    }
                    _ => return None,
                }
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
