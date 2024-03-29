use windows::w;
use windows::core::PCWSTR;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    System::Console::SetConsoleTitleW,
    System::SystemServices::SS_NOPREFIX,
    UI::{
        Controls::{EM_SCROLLCARET, EM_SETSEL},
        Input::KeyboardAndMouse::EnableWindow,
        WindowsAndMessaging as wm,
    },
};

use std::mem;
use std::ops::Range;

pub use crate::wide_string::*;

pub fn create_static_window(window_wnd: HWND, name: Option<&WideString>) -> HWND {
    unsafe {
        wm::CreateWindowExW(
            wm::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            PCWSTR::from_raw(name.map(WideString::as_ptr).unwrap_or(&mut 0u16)),
            wm::WS_CHILD | wm::WS_VISIBLE | wm::WINDOW_STYLE(wm::ES_CENTER as u32 | SS_NOPREFIX.0),
            0,
            0,
            0,
            0,
            window_wnd,
            wm::HMENU(0),
            HINSTANCE(0),
            None,
        )
    }
}

pub fn create_button_window(window_wnd: HWND, name: PCWSTR) -> HWND {
    unsafe {
        wm::CreateWindowExW(
            wm::WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            name,
            wm::WS_TABSTOP
                | wm::WS_VISIBLE
                | wm::WS_CHILD
                | wm::WINDOW_STYLE(wm::BS_CENTER as u32 | wm::BS_PUSHBUTTON as u32),
            0,
            0,
            0,
            0,
            window_wnd,
            wm::HMENU(0),
            HINSTANCE(0),
            None,
        )
    }
}

pub fn create_edit_window(window_wnd: HWND, style: wm::WINDOW_STYLE) -> HWND {
    // https://msdn.microsoft.com/en-us/library/windows/desktop/hh298433.aspx
    unsafe {
        wm::CreateWindowExW(
            wm::WS_EX_CLIENTEDGE,
            w!("EDIT"),
            PCWSTR(&mut 0u16),
            wm::WS_TABSTOP
                | wm::WS_CHILD
                | wm::WS_VISIBLE
                | wm::WS_BORDER
                | style
                | wm::WINDOW_STYLE(wm::ES_LEFT as u32 | wm::ES_NOHIDESEL as u32),
            0,
            0,
            0,
            0,
            window_wnd,
            wm::HMENU(0),
            HINSTANCE(0),
            None,
        )
    }
}

pub fn get_message() -> Option<wm::MSG> {
    let mut msg: wm::MSG = unsafe { mem::zeroed() };
    if unsafe { wm::GetMessageW(&mut msg, HWND(0), 0, 0) } != true {
        return None;
    }
    Some(msg)
}

pub fn enable_window(h_wnd: HWND, enable: bool) -> bool {
    unsafe { EnableWindow(h_wnd, enable).into() }
}

pub fn set_console_title(title: &WideString) -> bool {
    unsafe { SetConsoleTitleW(PCWSTR::from_raw(title.as_ptr())).into() }
}

pub fn set_window_text(h_wnd: HWND, wide: &WideString) -> bool {
    unsafe { wm::SetWindowTextW(h_wnd, PCWSTR::from_raw(wide.as_ptr())).into() }
}

pub fn get_window_text_length(h_wnd: HWND) -> i32 {
    unsafe { wm::GetWindowTextLengthW(h_wnd) }
}

pub fn get_window_text(h_wnd: HWND) -> WideString {
    let mut buf = vec![0u16; get_window_text_length(h_wnd) as usize + 1];
    let len = unsafe { wm::GetWindowTextW(h_wnd, &mut buf) };
    buf.truncate(len as usize + 1);
    WideString::from_raw(buf)
}

pub fn destroy_window(h_wnd: HWND) {
    unsafe {
        wm::DestroyWindow(h_wnd);
    }
}

pub fn close() {
    unsafe { wm::PostQuitMessage(0) }
}

pub fn set_edit_selection(h_wnd: HWND, celec: &Range<usize>) -> LRESULT {
    unsafe {
        wm::SendMessageW(
            h_wnd,
            EM_SETSEL,
            WPARAM(celec.start),
            LPARAM(celec.end.try_into().unwrap()),
        )
    }
}

pub fn set_edit_scroll_caret(h_wnd: HWND) -> LRESULT {
    unsafe { wm::SendMessageW(h_wnd, EM_SCROLLCARET, WPARAM(0), LPARAM(0)) }
}

pub fn get_client_rect(h_wnd: HWND) -> RECT {
    let mut rec: RECT = unsafe { mem::zeroed() };
    unsafe { wm::GetClientRect(h_wnd, &mut rec) };
    rec
}

pub fn move_window(h_wnd: HWND, rect: &RECT) -> bool {
    unsafe { wm::MoveWindow(h_wnd, rect.left, rect.top, rect.right, rect.bottom, true).into() }
}

pub fn is_window_visible(h_wnd: HWND) -> bool {
    unsafe { wm::IsWindowVisible(h_wnd).into() }
}

pub fn show_window(h_wnd: HWND, n_cmd_show: wm::SHOW_WINDOW_CMD) -> bool {
    unsafe { wm::ShowWindow(h_wnd, n_cmd_show).into() }
}

pub fn toggle_window_visible(h_wnd: HWND) -> bool {
    if is_window_visible(h_wnd) {
        show_window(h_wnd, wm::SW_HIDE)
    } else {
        show_window(h_wnd, wm::SW_SHOW)
    }
}

// rect utilities
pub trait RectUtil
where
    Self: Sized,
{
    fn inset(self, _: i32) -> Self;
    fn shift_down(self, delta: i32) -> Self;
    fn shift_right(self, delta: i32) -> Self;
    fn split_columns(self, at: i32) -> (Self, Self);
    fn split_rows(self, at: i32) -> (Self, Self);
}

impl RectUtil for RECT {
    fn inset(mut self, delta: i32) -> Self {
        self.left += delta;
        self.top += delta;
        self.right -= 2 * delta;
        self.bottom -= 2 * delta;
        self
    }
    fn shift_down(mut self, delta: i32) -> Self {
        self.top += delta;
        self.bottom -= delta;
        self
    }
    fn shift_right(mut self, delta: i32) -> Self {
        self.left += delta;
        self.right -= delta;
        self
    }
    fn split_columns(mut self, at: i32) -> (Self, Self) {
        let r = self.shift_right(at);
        self.right = at;
        (self, r)
    }
    fn split_rows(mut self, at: i32) -> (Self, Self) {
        let b = self.shift_down(at);
        self.bottom = at;
        (self, b)
    }
}

#[cfg(test)]
mod rect_util_tests {
    use super::*;

    #[test]
    fn inset() {
        let start = RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }
        .inset(10);
        assert_eq!(start.top, 110);
        assert_eq!(start.bottom, 80);
        assert_eq!(start.left, 110);
        assert_eq!(start.right, 80);
    }

    #[test]
    fn shift_down() {
        let start = RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }
        .shift_down(10);
        assert_eq!(start.top, 110);
        assert_eq!(start.bottom, 90);
        assert_eq!(start.left, 100);
        assert_eq!(start.right, 100);
    }

    #[test]
    fn shift_right() {
        let start = RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }
        .shift_right(10);
        assert_eq!(start.top, 100);
        assert_eq!(start.bottom, 100);
        assert_eq!(start.left, 110);
        assert_eq!(start.right, 90);
    }

    #[test]
    fn split_columns() {
        let start = RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }
        .split_columns(10);
        assert_eq!(start.0.top, 100);
        assert_eq!(start.0.bottom, 100);
        assert_eq!(start.0.left, 100);
        assert_eq!(start.0.right, 10);
        assert_eq!(start.1.top, 100);
        assert_eq!(start.1.bottom, 100);
        assert_eq!(start.1.left, 110);
        assert_eq!(start.1.right, 90);
    }

    #[test]
    fn split_rows() {
        let start = RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }
        .split_rows(10);
        assert_eq!(start.0.top, 100);
        assert_eq!(start.0.bottom, 10);
        assert_eq!(start.0.left, 100);
        assert_eq!(start.0.right, 100);
        assert_eq!(start.1.top, 110);
        assert_eq!(start.1.bottom, 90);
        assert_eq!(start.1.left, 100);
        assert_eq!(start.1.right, 100);
    }
}

// window's proc related function

#[cfg(target_arch = "x86_64")]
pub fn get_window_wrapper<'a, T>(h_wnd: HWND) -> Option<&'a mut T> {
    let ptr = unsafe { wm::GetWindowLongPtrW(h_wnd, wm::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86")]
pub fn get_window_wrapper<'a, T>(h_wnd: HWND) -> Option<&'a mut T> {
    let ptr = unsafe { wm::GetWindowLongW(h_wnd, wm::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86_64")]
pub fn set_window_wrapper(h_wnd: HWND, l_param: LPARAM) {
    let data = unsafe { &mut *(l_param.0 as *mut wm::CREATESTRUCTW) };
    unsafe {
        wm::SetWindowLongPtrW(h_wnd, wm::GWLP_USERDATA, data.lpCreateParams as isize);
    }
}

#[cfg(target_arch = "x86")]
pub fn set_window_wrapper(h_wnd: HWND, l_param: LPARAM) {
    let data = unsafe { &mut *(l_param as *mut wm::CREATESTRUCTW) };
    unsafe {
        wm::SetWindowLongW(h_wnd, wm::GWLP_USERDATA, data.lpCreateParams);
    }
}

pub trait Windowed {
    fn window_proc(&mut self, msg: u32, w_param: WPARAM, l_param: LPARAM) -> Option<LRESULT>;
}

pub unsafe extern "system" fn window_proc_generic<T: Windowed>(
    h_wnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == wm::WM_CREATE {
        set_window_wrapper(h_wnd, l_param);
    }
    // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param);
    if let Some(this) = get_window_wrapper::<T>(h_wnd) {
        if let Some(out) = this.window_proc(msg, w_param, l_param) {
            return out;
        }
    }
    wm::DefWindowProcW(h_wnd, msg, w_param, l_param)
}
