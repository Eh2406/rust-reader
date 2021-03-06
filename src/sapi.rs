use average::{Estimate, Variance};
use chrono;
use std::mem::size_of;
use std::mem::zeroed;
use winapi;
use winapi::shared::minwindef;
use winapi::shared::windef;
use winapi::um::libloaderapi;
use winapi::um::shellapi;
use winapi::um::winnt;
use winapi::um::winuser;

use std::cmp::{max, min};
use std::mem;
use std::ops::Range;
use std::ptr::null_mut;
use std::time::Instant;

use crate::window::*;

pub const WM_SAPI_EVENT: minwindef::UINT = winuser::WM_APP + 15;
pub const WM_APP_NOTIFICATION_ICON: minwindef::UINT = winuser::WM_APP + 16;

pub struct Com {
    hr: winnt::HRESULT,
}

impl Com {
    pub fn new() -> Com {
        println!("new for Com");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms678543.aspx
        let hr = unsafe { winapi::um::objbase::CoInitialize(null_mut()) };
        if failed(hr) {
            panic!("failed for Com");
        }
        Com { hr }
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms688715.aspx
        if self.hr != winapi::shared::winerror::RPC_E_CHANGED_MODE {
            unsafe { winapi::um::combaseapi::CoUninitialize() };
        }
        println!("drop for Com");
    }
}

pub struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602.aspx
    voice: &'a mut winapi::um::sapi51::ISpVoice,
    window: windef::HWND,
    edit: windef::HWND,
    rate: windef::HWND,
    reload_settings: windef::HWND,
    nicon: shellapi::NOTIFYICONDATAW,
    last_read: WideString,
    last_update: Option<(Instant, Range<usize>)>,
    us_per_utf16: [Variance; 21],
}

impl<'a> SpVoice<'a> {
    pub fn new<'c: 'a>(_con: &'c Com) -> Box<SpVoice<'a>> {
        println!("new for SpVoice");
        use winapi::Interface;
        let mut voice: *mut winapi::um::sapi51::ISpVoice = null_mut();
        let mut clsid_spvoice: winapi::shared::guiddef::CLSID = unsafe { mem::zeroed() };
        let sapi_id: WideString = "SAPI.SpVoice".into();

        unsafe {
            if failed(winapi::um::combaseapi::CLSIDFromProgID(
                sapi_id.as_ptr(),
                &mut clsid_spvoice,
            )) {
                panic!("failed for SpVoice at CLSIDFromProgID");
            }

            if failed(winapi::um::combaseapi::CoCreateInstance(
                &clsid_spvoice,
                null_mut(),
                winapi::um::combaseapi::CLSCTX_ALL,
                &winapi::um::sapi51::ISpVoice::uuidof(),
                &mut voice as *mut *mut winapi::um::sapi51::ISpVoice
                    as *mut *mut winapi::ctypes::c_void,
            )) {
                panic!("failed for SpVoice at CoCreateInstance");
            }
            let mut out = Box::new(SpVoice {
                voice: &mut *voice,
                window: null_mut(),
                edit: null_mut(),
                rate: null_mut(),
                reload_settings: null_mut(),
                nicon: zeroed(),
                last_read: WideString::new(),
                last_update: None,
                us_per_utf16: Default::default(),
            });

            let window_class_name: WideString = "SAPI_event_window_class_name".into();
            winuser::RegisterClassW(&winuser::WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc_generic::<SpVoice>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: null_mut(),
                hIcon: winuser::LoadIconW(
                    libloaderapi::GetModuleHandleW(null_mut()),
                    winuser::MAKEINTRESOURCEW(1),
                ),
                hCursor: winuser::LoadCursorW(null_mut(), winuser::IDI_APPLICATION),
                hbrBackground: 16 as windef::HBRUSH,
                lpszMenuName: null_mut(),
                lpszClassName: window_class_name.as_ptr(),
            });
            out.window = winuser::CreateWindowExW(
                0,
                window_class_name.as_ptr(),
                &0u16,
                winuser::WS_OVERLAPPEDWINDOW | winuser::WS_CLIPSIBLINGS | winuser::WS_CLIPCHILDREN,
                0,
                0,
                0,
                0,
                winuser::GetDesktopWindow(),
                null_mut(),
                null_mut(),
                &mut *out as *mut _ as minwindef::LPVOID,
            );

            out.nicon.cbSize = size_of::<shellapi::NOTIFYICONDATAW>() as u32;
            out.nicon.hWnd = out.window;
            out.nicon.uCallbackMessage = WM_APP_NOTIFICATION_ICON;
            out.nicon.uID = 1 as u32;
            out.nicon.uFlags |= shellapi::NIF_ICON;
            out.nicon.hIcon = winuser::LoadIconW(
                libloaderapi::GetModuleHandleW(null_mut()),
                winuser::MAKEINTRESOURCEW(1),
            );
            out.nicon.uFlags |= shellapi::NIF_MESSAGE;
            *out.nicon.u.uVersion_mut() = shellapi::NOTIFYICON_VERSION_4;
            let err = shellapi::Shell_NotifyIconW(shellapi::NIM_ADD, &mut out.nicon);
            if err == 0 {
                panic!("failed for Shell_NotifyIconW NIM_ADD");
            }

            let err = shellapi::Shell_NotifyIconW(shellapi::NIM_SETVERSION, &mut out.nicon);
            if err == 0 {
                panic!("failed for Shell_NotifyIconW ");
            }

            out.edit = create_edit_window(
                out.window,
                winuser::WS_VSCROLL | winuser::ES_MULTILINE | winuser::ES_AUTOVSCROLL,
            );
            out.rate = create_static_window(out.window, None);
            out.reload_settings = create_button_window(out.window, Some(&"Show Settings".into()));
            move_window(
                out.window,
                &windef::RECT {
                    left: 0,
                    top: 0,
                    right: 400,
                    bottom: 400,
                },
            );
            out.set_notify_window_message();
            out.set_volume(100);
            out.set_alert_boundary(winapi::um::sapi51::SPEI_PHONEME);
            out.set_interest(&[5, 1, 2], &[]);
            out
        }
    }

    #[allow(dead_code)]
    pub fn get_window_handle(&mut self) -> windef::HWND {
        self.window
    }

    pub fn set_time_estimater(&mut self, t: [Variance; 21]) {
        self.us_per_utf16 = t;
    }

    pub fn get_time_estimater(&self) -> [Variance; 21] {
        self.us_per_utf16.clone()
    }

    pub fn toggle_window_visible(&self) -> minwindef::BOOL {
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
        unsafe { self.voice.WaitUntilDone(winapi::um::winbase::INFINITE) };
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

    pub fn set_alert_boundary(&mut self, boundary: winapi::um::sapi51::SPEVENTENUM) {
        unsafe { self.voice.SetAlertBoundary(boundary) };
    }

    #[allow(dead_code)]
    pub fn get_alert_boundary(&mut self) -> winapi::um::sapi51::SPEVENTENUM {
        let mut boundary = 0;
        unsafe { self.voice.GetAlertBoundary(&mut boundary) };
        boundary
    }

    pub fn get_status(&mut self) -> winapi::um::sapi51::SPVOICESTATUS {
        let mut status: winapi::um::sapi51::SPVOICESTATUS = unsafe { mem::zeroed() };
        unsafe { self.voice.GetStatus(&mut status, null_mut()) };
        status
    }

    fn set_notify_window_message(&mut self) {
        unsafe {
            self.voice
                .SetNotifyWindowMessage(self.window, WM_SAPI_EVENT, 0, 0)
        };
    }

    pub fn set_interest(&mut self, event: &[u32], queued: &[u32]) {
        let queued = queued
            .iter()
            .map(|&x| winapi::um::sapi51::SPFEI(x))
            .fold(0u64, |acc, x| acc | x);
        let event = event
            .iter()
            .map(|&x| winapi::um::sapi51::SPFEI(x))
            .fold(queued, |acc, x| acc | x);
        unsafe { self.voice.SetInterest(event, queued) };
    }
}

fn format_duration(d: chrono::Duration) -> String {
    let h = d.num_hours();
    let m = d.num_minutes() - d.num_hours() * 60;
    let s = d.num_seconds() - d.num_minutes() * 60;
    if d.num_hours() == 0 {
        format!("{}:{:0>#2}", m, s)
    } else {
        format!("{}:{:0>#2}:{:0>#2}", h, m, s)
    }
}

#[test]
fn test_format_duration() {
    let duration = chrono::Duration::hours(25);
    assert_eq!(format_duration(duration), "25:00:00");
    let duration = chrono::Duration::hours(1) + chrono::Duration::seconds(1);
    assert_eq!(format_duration(duration), "1:00:01");
    let duration = chrono::Duration::hours(1) - chrono::Duration::seconds(1);
    assert_eq!(format_duration(duration), "59:59");
    let duration = chrono::Duration::seconds(61);
    assert_eq!(format_duration(duration), "1:01");
    let duration = chrono::Duration::seconds(60);
    assert_eq!(format_duration(duration), "1:00");
    let duration = chrono::Duration::seconds(59);
    assert_eq!(format_duration(duration), "0:59");
    let duration = chrono::Duration::seconds(9);
    assert_eq!(format_duration(duration), "0:09");
    let duration = chrono::Duration::seconds(0);
    assert_eq!(format_duration(duration), "0:00");
}

impl<'a> Windowed for SpVoice<'a> {
    fn window_proc(
        &mut self,
        msg: minwindef::UINT,
        w_param: minwindef::WPARAM,
        l_param: minwindef::LPARAM,
    ) -> Option<minwindef::LRESULT> {
        match msg {
            winuser::WM_DESTROY | winuser::WM_QUERYENDSESSION | winuser::WM_ENDSESSION => close(),
            WM_SAPI_EVENT => {
                let status = self.get_status();
                let word_range = status.word_range();
                let rate = self.get_rate();
                if word_range.end == 0 {
                    // called before start of reading.
                    self.last_update = None;
                    return Some(0);
                }
                if status.dwRunningState == 3 {
                    // called before end of reading.
                    let window_title = "100.0% 0:00 rust_reader".into();
                    set_console_title(&window_title);
                    set_window_text(self.window, &window_title);
                    self.last_update = None;
                    return Some(0);
                }
                if let Some((ref old_time, ref old_word_range)) = self.last_update {
                    if old_word_range.start == word_range.start {
                        return Some(0);
                    }
                    let elapsed = chrono::Duration::from_std(old_time.elapsed())
                        .expect("bad time diffrence.")
                        .num_microseconds()
                        .expect("bad time diffrence.");
                    let new_rate =
                        (elapsed as f64) / ((word_range.start - old_word_range.start) as f64);
                    self.us_per_utf16[rate as usize + 10].add(new_rate);
                }
                self.last_update = Some((Instant::now(), word_range.clone()));
                let len_left = (self.last_read.len() - word_range.end) as f64;
                let ms_left = len_left * self.us_per_utf16[rate as usize + 10].mean()
                    + (len_left * self.us_per_utf16[rate as usize + 10].sample_variance()).sqrt();
                let window_title = format!(
                    "{:.1}% {} \"{}\" rust_reader",
                    100.0 * (word_range.start as f64) / (self.last_read.len() as f64),
                    format_duration(chrono::Duration::microseconds(ms_left as i64)),
                    self.last_read.get_slice(word_range.clone())
                )
                .into();
                set_console_title(&window_title);
                set_window_text(self.window, &window_title);
                set_edit_selection(self.edit, &word_range);
                set_edit_scroll_caret(self.edit);
                return Some(0);
            }
            winuser::WM_SIZE => {
                let rect = get_client_rect(self.window);
                if (w_param <= 2) && rect.right > 0 && rect.bottom > 0 {
                    let (up, down) = rect.inset(3).split_rows(25);
                    move_window(self.edit, &down.inset(3));
                    let (left, right) = up.split_columns(120);
                    move_window(self.reload_settings, &left.inset(3));
                    unsafe {
                        winuser::InvalidateRect(self.rate, null_mut(), minwindef::TRUE);
                    }
                    move_window(self.rate, &right.inset(3));
                    return Some(0);
                }
            }
            winuser::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param as *mut winuser::MINMAXINFO) };
                data.ptMinTrackSize.x = 300;
                data.ptMinTrackSize.y = 110;
                return Some(0);
            }
            winuser::WM_COMMAND => {
                use crate::press_hotkey;
                use crate::Action;
                if self.reload_settings as isize == l_param
                    && minwindef::HIWORD(w_param as u32) == winuser::BN_CLICKED
                {
                    press_hotkey(Action::ShowSettings);
                    return Some(0);
                }
            }
            WM_APP_NOTIFICATION_ICON => {
                if minwindef::LOWORD(l_param as u32) == (winuser::WM_LBUTTONUP as u16) {
                    self.toggle_window_visible();
                    return Some(0);
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
        unsafe { shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.nicon) };
        println!("drop for SpVoice");
    }
}

pub trait StatusUtil {
    fn word_range(&self) -> Range<usize>;
    fn sent_range(&self) -> Range<usize>;
}

impl StatusUtil for winapi::um::sapi51::SPVOICESTATUS {
    fn word_range(&self) -> Range<usize> {
        self.ulInputWordPos as usize..(self.ulInputWordPos + self.ulInputWordLen) as usize
    }
    fn sent_range(&self) -> Range<usize> {
        self.ulInputSentPos as usize..(self.ulInputSentPos + self.ulInputSentLen) as usize
    }
}
