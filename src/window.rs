use winapi;
use winapi::shared::minwindef;
use winapi::shared::windef;
use winapi::um::winnt;
use winapi::um::winuser;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging;

use std::mem;
use std::ops::Range;
use std::ptr::null_mut;

pub use crate::wide_string::*;

pub fn create_static_window(window_wnd: windef::HWND, name: Option<&WideString>) -> windef::HWND {
    let wide_static: WideString = "STATIC".into();
    unsafe {
        winuser::CreateWindowExW(
            0,
            wide_static.as_ptr(),
            name.map(WideString::as_ptr).unwrap_or(&0u16),
            winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::SS_CENTER | winuser::SS_NOPREFIX,
            0,
            0,
            0,
            0,
            window_wnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    }
}

pub fn create_button_window(window_wnd: windef::HWND, name: Option<&WideString>) -> windef::HWND {
    let wide_button: WideString = "BUTTON".into();
    unsafe {
        winuser::CreateWindowExW(
            0,
            wide_button.as_ptr(),
            name.map(WideString::as_ptr).unwrap_or(&0u16),
            winuser::WS_TABSTOP | winuser::BS_CENTER | winuser::WS_VISIBLE | winuser::WS_CHILD
                | winuser::BS_PUSHBUTTON,
            0,
            0,
            0,
            0,
            window_wnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    }
}

pub fn create_edit_window(window_wnd: windef::HWND, style: minwindef::DWORD) -> windef::HWND {
    // https://msdn.microsoft.com/en-us/library/windows/desktop/hh298433.aspx
    let wide_edit: WideString = "EDIT".into();
    unsafe {
        winuser::CreateWindowExW(
            winuser::WS_EX_CLIENTEDGE,
            wide_edit.as_ptr(),
            &0u16,
            winuser::WS_TABSTOP | winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::WS_BORDER
                | winuser::ES_LEFT | winuser::ES_NOHIDESEL | style,
            0,
            0,
            0,
            0,
            window_wnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    }
}

#[inline]
pub fn failed(hr: winnt::HRESULT) -> bool {
    hr < 0
}

#[inline]
#[allow(dead_code)]
pub fn succeeded(hr: winnt::HRESULT) -> bool {
    !failed(hr)
}

pub fn get_message() -> Option<WindowsAndMessaging::MSG> {
        let mut msg: WindowsAndMessaging::MSG = unsafe { mem::zeroed() };
        if unsafe { WindowsAndMessaging::GetMessageW(&mut msg, HWND(0), 0, 0) } != true {
            return None;
    }
    Some(msg)
}

pub fn enable_window(h_wnd: windef::HWND, enable: bool) -> minwindef::BOOL {
    unsafe { winuser::EnableWindow(h_wnd, enable as minwindef::BOOL) }
}

pub fn set_console_title(title: &WideString) -> i32 {
    unsafe { winapi::um::wincon::SetConsoleTitleW(title.as_ptr()) }
}

pub fn set_window_text(h_wnd: windef::HWND, wide: &WideString) -> minwindef::BOOL {
    unsafe { winuser::SetWindowTextW(h_wnd, wide.as_ptr()) }
}

pub fn get_window_text_length(h_wnd: windef::HWND) -> minwindef::INT {
    unsafe { winuser::GetWindowTextLengthW(h_wnd) }
}

pub fn get_window_text(h_wnd: windef::HWND) -> WideString {
    let mut buf = vec![0u16; get_window_text_length(h_wnd) as usize + 1];
    let len = unsafe { winuser::GetWindowTextW(h_wnd, buf.as_mut_ptr(), buf.len() as i32) };
    buf.truncate(len as usize + 1);
    WideString::from_raw(buf)
}

pub fn destroy_window(h_wnd: windef::HWND) {
    unsafe {
        winuser::DestroyWindow(h_wnd);
    }
}

pub fn close() {
    unsafe { winuser::PostQuitMessage(0) }
}

pub fn set_edit_selection(h_wnd: windef::HWND, celec: &Range<usize>) -> minwindef::LRESULT {
    unsafe {
        winuser::SendMessageW(
            h_wnd,
            minwindef::UINT::from(winuser::EM_SETSEL),
            celec.start as minwindef::WPARAM,
            celec.end as minwindef::LPARAM,
        )
    }
}

pub fn set_edit_scroll_caret(h_wnd: windef::HWND) -> minwindef::LRESULT {
    unsafe { winuser::SendMessageW(h_wnd, minwindef::UINT::from(winuser::EM_SCROLLCARET), 0, 0) }
}

pub fn get_client_rect(h_wnd: windef::HWND) -> windef::RECT {
    let mut rec: windef::RECT = unsafe { mem::zeroed() };
    unsafe { winuser::GetClientRect(h_wnd, &mut rec) };
    rec
}

pub fn move_window(h_wnd: windef::HWND, rect: &windef::RECT) -> minwindef::BOOL {
    unsafe {
        winuser::MoveWindow(
            h_wnd,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            minwindef::TRUE,
        )
    }
}

pub fn is_window_visible(h_wnd: windef::HWND) -> minwindef::BOOL {
    unsafe { winuser::IsWindowVisible(h_wnd) }
}

pub fn show_window(h_wnd: windef::HWND, n_cmd_show: winapi::ctypes::c_int) -> minwindef::BOOL {
    unsafe { winuser::ShowWindow(h_wnd, n_cmd_show) }
}

pub fn toggle_window_visible(h_wnd: windef::HWND) -> minwindef::BOOL {
    if 1 == is_window_visible(h_wnd) {
        show_window(h_wnd, winuser::SW_HIDE)
    } else {
        show_window(h_wnd, winuser::SW_SHOW)
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

impl RectUtil for windef::RECT {
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
        let start = windef::RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }.inset(10);
        assert_eq!(start.top, 110);
        assert_eq!(start.bottom, 80);
        assert_eq!(start.left, 110);
        assert_eq!(start.right, 80);
    }

    #[test]
    fn shift_down() {
        let start = windef::RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }.shift_down(10);
        assert_eq!(start.top, 110);
        assert_eq!(start.bottom, 90);
        assert_eq!(start.left, 100);
        assert_eq!(start.right, 100);
    }

    #[test]
    fn shift_right() {
        let start = windef::RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }.shift_right(10);
        assert_eq!(start.top, 100);
        assert_eq!(start.bottom, 100);
        assert_eq!(start.left, 110);
        assert_eq!(start.right, 90);
    }

    #[test]
    fn split_columns() {
        let start = windef::RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }.split_columns(10);
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
        let start = windef::RECT {
            bottom: 100,
            left: 100,
            right: 100,
            top: 100,
        }.split_rows(10);
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
pub fn get_window_wrapper<'a, T>(h_wnd: windef::HWND) -> Option<&'a mut T> {
    let ptr = unsafe { winuser::GetWindowLongPtrW(h_wnd, winuser::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86")]
pub fn get_window_wrapper<'a, T>(h_wnd: windef::HWND) -> Option<&'a mut T> {
    let ptr = unsafe { winuser::GetWindowLongW(h_wnd, winuser::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86_64")]
pub fn set_window_wrapper(h_wnd: windef::HWND, l_param: minwindef::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winuser::CREATESTRUCTW) };
    unsafe {
        winuser::SetWindowLongPtrW(
            h_wnd,
            winuser::GWLP_USERDATA,
            data.lpCreateParams as winapi::shared::basetsd::LONG_PTR,
        );
    }
}

#[cfg(target_arch = "x86")]
pub fn set_window_wrapper(h_wnd: windef::HWND, l_param: minwindef::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winuser::CREATESTRUCTW) };
    unsafe {
        winuser::SetWindowLongW(
            h_wnd,
            winuser::GWLP_USERDATA,
            data.lpCreateParams as winnt::LONG,
        );
    }
}

pub trait Windowed {
    fn window_proc(
        &mut self,
        msg: minwindef::UINT,
        w_param: minwindef::WPARAM,
        l_param: minwindef::LPARAM,
    ) -> Option<minwindef::LRESULT>;
}

pub unsafe extern "system" fn window_proc_generic<T: Windowed>(
    h_wnd: windef::HWND,
    msg: minwindef::UINT,
    w_param: minwindef::WPARAM,
    l_param: minwindef::LPARAM,
) -> minwindef::LRESULT {
    if msg == winuser::WM_CREATE {
        set_window_wrapper(h_wnd, l_param);
    }
    // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param);
    if let Some(this) = get_window_wrapper::<T>(h_wnd) {
        if let Some(out) = this.window_proc(msg, w_param, l_param) {
            return out;
        }
    }
    winuser::DefWindowProcW(h_wnd, msg, w_param, l_param)
}
