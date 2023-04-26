use average::{Estimate, Variance};
use chrono;
use std::mem::size_of;
use std::mem::{MaybeUninit, zeroed};

use windows::core::PCWSTR;
use windows::w;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi,
    Media::Speech,
    System::Com as syscom,
    System::LibraryLoader,
    System::WindowsProgramming::INFINITE,
    UI::Shell,
    UI::WindowsAndMessaging as wm,
};

use std::cmp::{max, min};
use std::mem;
use std::ops::Range;
use std::ptr::null_mut;
use std::time::Instant;

use crate::window::*;

pub const WM_SAPI_EVENT: u32 = wm::WM_APP + 15;
pub const WM_APP_NOTIFICATION_ICON: u32 = wm::WM_APP + 16;

pub struct Com {}

impl Com {
    pub fn new() -> Com {
        println!("new for Com");
        match unsafe { syscom::CoInitialize(Some(null_mut())) } {
            Ok(_) => Com {},
            Err(_) => panic!("failed for Com"),
        }
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        unsafe { syscom::CoUninitialize() };
        println!("drop for Com");
    }
}

pub struct SpVoice {
    // https://msdn.microsoft.com/en-us/library/ms723602.aspx
    voice: Speech::ISpVoice,
    window: HWND,
    edit: HWND,
    rate: HWND,
    reload_settings: HWND,
    nicon: Shell::NOTIFYICONDATAW,
    last_read: WideString,
    last_update: Option<(Instant, Range<usize>)>,
    us_per_utf16: [Variance; 21],
}

impl SpVoice {
    pub fn new<'c>(_con: &'c Com) -> Box<SpVoice> {
        println!("new for SpVoice");

        unsafe {
            let mut out = Box::new(SpVoice {
                voice: syscom::CoCreateInstance(&Speech::SpVoice, None, syscom::CLSCTX_ALL)
                    .expect("failed for SpVoice at CoCreateInstance"),
                window: HWND(0),
                edit: HWND(0),
                rate: HWND(0),
                reload_settings: HWND(0),
                nicon: zeroed(),
                last_read: WideString::new(),
                last_update: None,
                us_per_utf16: Default::default(),
            });

            let window_class_name = w!("SAPI_event_window_class_name");
            wm::RegisterClassW(&wm::WNDCLASSW {
                style: wm::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(window_proc_generic::<SpVoice>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: HINSTANCE(0),
                hIcon: wm::LoadIconW(
                    LibraryLoader::GetModuleHandleW(PCWSTR::null()).unwrap(),
                    PCWSTR::from_raw(1 as *const u16),
                )
                .expect("failed to load icon"),
                hCursor: wm::LoadCursorW(HINSTANCE(0), wm::IDI_APPLICATION)
                    .expect("failed to load icon"),
                hbrBackground: Gdi::HBRUSH(16),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: window_class_name,
            });
            out.window = wm::CreateWindowExW(
                wm::WINDOW_EX_STYLE(0),
                window_class_name,
                PCWSTR(&mut 0u16),
                wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPSIBLINGS | wm::WS_CLIPCHILDREN,
                0,
                0,
                0,
                0,
                wm::GetDesktopWindow(),
                wm::HMENU(0),
                HINSTANCE(0),
                Some(&mut *out as *mut _ as _),
            );

            out.nicon.cbSize = size_of::<Shell::NOTIFYICONDATAW>() as u32;
            out.nicon.hWnd = out.window;
            out.nicon.uCallbackMessage = WM_APP_NOTIFICATION_ICON;
            out.nicon.uID = 1 as u32;
            out.nicon.uFlags |= Shell::NIF_ICON;
            out.nicon.hIcon = wm::LoadIconW(
                LibraryLoader::GetModuleHandleW(PCWSTR::null()).unwrap(),
                PCWSTR::from_raw(1 as *const u16),
            )
            .expect("failed to load icon");
            out.nicon.uFlags |= Shell::NIF_MESSAGE;
            out.nicon.Anonymous.uVersion = Shell::NOTIFYICON_VERSION_4;
            let err = Shell::Shell_NotifyIconW(Shell::NIM_ADD, &mut out.nicon);
            if err == false {
                panic!("failed for Shell_NotifyIconW NIM_ADD");
            }

            let err = Shell::Shell_NotifyIconW(Shell::NIM_SETVERSION, &mut out.nicon);
            if err == false {
                panic!("failed for Shell_NotifyIconW ");
            }

            out.edit = create_edit_window(
                out.window,
                wm::WS_VSCROLL
                    | wm::WINDOW_STYLE(wm::ES_MULTILINE as u32 | wm::ES_AUTOVSCROLL as u32),
            );
            out.rate = create_static_window(out.window, None);
            out.reload_settings = create_button_window(out.window, w!("Show Settings"));
            move_window(
                out.window,
                &RECT {
                    left: 0,
                    top: 0,
                    right: 400,
                    bottom: 400,
                },
            );
            out.set_notify_window_message();
            out.set_volume(100);
            out.set_alert_boundary(Speech::SPEI_PHONEME);
            out.set_interest(
                &[
                    Speech::SPEI_WORD_BOUNDARY,
                    Speech::SPEI_START_INPUT_STREAM,
                    Speech::SPEI_END_INPUT_STREAM,
                ],
                &[],
            );
            out
        }
    }

    #[allow(dead_code)]
    pub fn get_window_handle(&mut self) -> HWND {
        self.window
    }

    pub fn set_time_estimater(&mut self, t: [Variance; 21]) {
        self.us_per_utf16 = t;
    }

    pub fn get_time_estimater(&self) -> [Variance; 21] {
        self.us_per_utf16.clone()
    }

    pub fn toggle_window_visible(&self) -> bool {
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
        unsafe {
            self.voice.Speak(
                PCWSTR::from_raw(self.last_read.as_ptr()),
                (Speech::SVSFlagsAsync.0 | Speech::SVSFPurgeBeforeSpeak.0 | Speech::SVSFIsNotXML.0)
                    .try_into()
                    .unwrap(),
                None,
            )
        }
        .unwrap();
        self.last_update = None;
    }

    pub fn wait(&mut self) {
        unsafe { self.voice.WaitUntilDone(INFINITE) }.unwrap();
    }

    pub fn speak_wait<T: Into<WideString>>(&mut self, string: T) {
        self.speak(string);
        self.wait();
    }

    pub fn pause(&mut self) {
        unsafe { self.voice.Pause() }.unwrap();
        self.last_update = None;
    }

    pub fn resume(&mut self) {
        unsafe { self.voice.Resume() }.unwrap();
        self.last_update = None;
    }

    pub fn set_rate(&mut self, rate: i32) -> i32 {
        let rate = max(min(rate, 10), -10);
        unsafe { self.voice.SetRate(rate) }.unwrap();
        self.last_update = None;
        self.get_rate()
    }

    pub fn get_rate(&mut self) -> i32 {
        let mut rate = 0;
        unsafe { self.voice.GetRate(&mut rate) }.unwrap();
        set_window_text(self.rate, &format!("reading at rate: {}", rate).into());
        rate
    }

    pub fn change_rate(&mut self, delta: i32) -> i32 {
        let rate = self.get_rate() + delta;
        self.set_rate(rate)
    }

    pub fn get_voice(&mut self) -> Option<String> {
        unsafe {
            self.voice
                .GetVoice()
                .ok()
                .and_then(|t| t.OpenKey(w!("Attributes")).ok())
                .and_then(|k| k.GetStringValue(w!("name")).ok())
                .and_then(|s| s.to_string().ok())
        }
    }

    pub fn set_voice(&mut self, token: Speech::ISpObjectToken) {
        unsafe { self.voice.SetVoice(&token).unwrap() };
    }

    pub fn get_all_voices(&mut self) -> Option<String> {
        unsafe {
            let category: Speech::ISpObjectTokenCategory =
                syscom::CoCreateInstance(&Speech::SpObjectTokenCategory, None, syscom::CLSCTX_ALL)
                    .expect("failed to get voice category");
            category.SetId(
                w!(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\Voices"),
                false,
            ).unwrap();

            let token_enum = category.EnumTokens(w!(""), w!("")).unwrap();
            loop {
                let mut token = MaybeUninit::uninit();
                token_enum.Next(1, token.as_mut_ptr(), None).unwrap();
                match token.assume_init() {
                    Some(t) => {
                        self.set_voice(t);
                        //println!("some voice: {}", t.OpenKey(w!("Attributes")).ok()
                        //    .and_then(|k| k.GetStringValue(w!("name")).ok())
                        //    .and_then(|s| s.to_string().ok())
                        //    .unwrap())
                    },
                    None => { break; },
                }
            }
        }
        Some("foo".to_string())
    }

    pub fn set_volume(&mut self, volume: u16) {
        unsafe { self.voice.SetVolume(min(volume, 100)) }.unwrap();
    }

    #[allow(dead_code)]
    pub fn get_volume(&mut self) -> u16 {
        let mut volume = 0;
        unsafe { self.voice.GetVolume(&mut volume) }.unwrap();
        volume
    }

    pub fn set_alert_boundary(&mut self, boundary: Speech::SPEVENTENUM) {
        unsafe { self.voice.SetAlertBoundary(boundary) }.unwrap();
    }

    #[allow(dead_code)]
    pub fn get_alert_boundary(&mut self) -> Speech::SPEVENTENUM {
        let mut boundary = Speech::SPEVENTENUM(0);
        unsafe { self.voice.GetAlertBoundary(&mut boundary) }.unwrap();
        boundary
    }

    pub fn get_status(&mut self) -> Speech::SPVOICESTATUS {
        let mut status: Speech::SPVOICESTATUS = unsafe { mem::zeroed() };
        unsafe { self.voice.GetStatus(&mut status, null_mut()) }.unwrap();
        status
    }

    fn set_notify_window_message(&mut self) {
        unsafe {
            self.voice
                .SetNotifyWindowMessage(self.window, WM_SAPI_EVENT, WPARAM(0), LPARAM(0))
        }
        .unwrap();
    }

    pub fn set_interest(&mut self, event: &[Speech::SPEVENTENUM], queued: &[Speech::SPEVENTENUM]) {
        let queued = queued
            .iter()
            .map(|&x| {
                (1 << x.0) | (1 << Speech::SPEI_RESERVED1.0) | (1 << Speech::SPEI_RESERVED2.0)
            })
            .fold(0u64, |acc, x| acc | x);
        let event = event
            .iter()
            .map(|&x| {
                (1 << x.0) | (1 << Speech::SPEI_RESERVED1.0) | (1 << Speech::SPEI_RESERVED2.0)
            })
            .fold(queued, |acc, x| acc | x);
        unsafe { self.voice.SetInterest(event, queued) }.unwrap();
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

impl Windowed for SpVoice {
    fn window_proc(&mut self, msg: u32, w_param: WPARAM, l_param: LPARAM) -> Option<LRESULT> {
        match msg {
            wm::WM_DESTROY | wm::WM_QUERYENDSESSION | wm::WM_ENDSESSION => close(),
            WM_SAPI_EVENT => {
                let status = self.get_status();
                // convert rate from range (-10, 10) to (0, 20)
                let rate_shifted = 10u32
                    .checked_add_signed(self.get_rate())
                    .expect("bad rate < -10") as usize;
                let word_range = status.word_range();
                if word_range.end == 0 {
                    // called before start of reading.
                    self.last_update = None;
                    return Some(LRESULT(0));
                }
                if status.dwRunningState == 3 {
                    // called before end of reading.
                    let window_title = "100.0% 0:00 rust_reader".into();
                    set_console_title(&window_title);
                    set_window_text(self.window, &window_title);
                    self.last_update = None;
                    return Some(LRESULT(0));
                }
                if let Some((ref old_time, ref old_word_range)) = self.last_update {
                    if old_word_range.start == word_range.start {
                        return Some(LRESULT(0));
                    }
                    let elapsed = chrono::Duration::from_std(old_time.elapsed())
                        .expect("bad time diffrence.")
                        .num_microseconds()
                        .expect("bad time diffrence.");
                    let new_rate =
                        (elapsed as f64) / ((word_range.start - old_word_range.start) as f64);
                    self.us_per_utf16[rate_shifted].add(new_rate);
                }
                self.last_update = Some((Instant::now(), word_range.clone()));
                let len_left = (self.last_read.len() - word_range.end) as f64;
                let ms_left = len_left * self.us_per_utf16[rate_shifted].mean()
                    + (len_left * self.us_per_utf16[rate_shifted].sample_variance()).sqrt();
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
                return Some(LRESULT(0));
            }
            wm::WM_SIZE => {
                let rect = get_client_rect(self.window);
                if (w_param.0 <= 2) && rect.right > 0 && rect.bottom > 0 {
                    let (up, down) = rect.inset(3).split_rows(25);
                    move_window(self.edit, &down.inset(3));
                    let (left, right) = up.split_columns(120);
                    move_window(self.reload_settings, &left.inset(3));
                    unsafe {
                        Gdi::InvalidateRect(self.rate, None, true);
                    }
                    move_window(self.rate, &right.inset(3));
                    return Some(LRESULT(0));
                }
            }
            wm::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param.0 as *mut u32 as *mut wm::MINMAXINFO) };
                data.ptMinTrackSize.x = 300;
                data.ptMinTrackSize.y = 110;
                return Some(LRESULT(0));
            }
            wm::WM_COMMAND => {
                use crate::press_hotkey;
                use crate::Action;
                if self.reload_settings.0 == l_param.0
                    && ((w_param.0 >> 16) & 0xffff) as u32 == wm::BN_CLICKED
                {
                    press_hotkey(Action::ShowSettings);
                    return Some(LRESULT(0));
                }
            }
            WM_APP_NOTIFICATION_ICON => {
                if (l_param.0 & 0xffff) as u32 == wm::WM_LBUTTONUP {
                    self.toggle_window_visible();
                    return Some(LRESULT(0));
                }
            }
            _ => {}
        }
        None
    }
}

impl Drop for SpVoice {
    fn drop(&mut self) {
        unsafe { Shell::Shell_NotifyIconW(Shell::NIM_DELETE, &mut self.nicon) };
        println!("drop for SpVoice");
    }
}

pub trait StatusUtil {
    fn word_range(&self) -> Range<usize>;
    fn sent_range(&self) -> Range<usize>;
}

impl StatusUtil for Speech::SPVOICESTATUS {
    fn word_range(&self) -> Range<usize> {
        self.ulInputWordPos as usize..(self.ulInputWordPos + self.ulInputWordLen) as usize
    }
    fn sent_range(&self) -> Range<usize> {
        self.ulInputSentPos as usize..(self.ulInputSentPos + self.ulInputSentLen) as usize
    }
}
