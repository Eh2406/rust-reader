use winapi;

use std::ops::Range;
use std::mem;

pub use wide_string::*;

// waiting for winapi
pub mod winapi_stub {
    #![allow(dead_code, non_snake_case)]
    use winapi::shared::windef::HMENU;

    pub const ID_EDITCHILD: HMENU = 100 as HMENU;
}

#[inline]
pub fn failed(hr: winapi::um::winnt::HRESULT) -> bool {
    hr < 0
}

#[inline]
#[allow(dead_code)]
pub fn succeeded(hr: winapi::um::winnt::HRESULT) -> bool {
    !failed(hr)
}

pub fn get_message() -> Option<winapi::um::winuser::MSG> {
    use std::ptr::null_mut;
    let mut msg: winapi::um::winuser::MSG = unsafe { mem::zeroed() };
    if unsafe { winapi::um::winuser::GetMessageW(&mut msg, null_mut(), 0, 0) } <= 0 {
        return None;
    }
    Some(msg)
}

pub fn set_console_title(title: &WideString) -> i32 {
    unsafe { winapi::um::wincon::SetConsoleTitleW(title.as_ptr()) }
}

pub fn set_window_text(
    h_wnd: winapi::shared::windef::HWND,
    wide: &WideString,
) -> winapi::shared::minwindef::BOOL {
    unsafe { winapi::um::winuser::SetWindowTextW(h_wnd, wide.as_ptr()) }
}

pub fn close() {
    unsafe { winapi::um::winuser::PostQuitMessage(0) }
}

pub fn set_edit_selection(
    h_wnd: winapi::shared::windef::HWND,
    celec: &Range<usize>,
) -> winapi::shared::minwindef::LRESULT {
    unsafe {
        winapi::um::winuser::SendMessageW(
            h_wnd,
            winapi::shared::minwindef::UINT::from(winapi::um::winuser::EM_SETSEL),
            celec.start as winapi::shared::minwindef::WPARAM,
            celec.end as winapi::shared::minwindef::LPARAM,
        )
    }
}

pub fn set_edit_scroll_caret(
    h_wnd: winapi::shared::windef::HWND,
) -> winapi::shared::minwindef::LRESULT {
    unsafe {
        winapi::um::winuser::SendMessageW(
            h_wnd,
            winapi::shared::minwindef::UINT::from(winapi::um::winuser::EM_SCROLLCARET),
            0 as winapi::shared::minwindef::WPARAM,
            0 as winapi::shared::minwindef::LPARAM,
        )
    }
}

pub fn get_client_rect(h_wnd: winapi::shared::windef::HWND) -> winapi::shared::windef::RECT {
    let mut rec: winapi::shared::windef::RECT = unsafe { mem::zeroed() };
    unsafe { winapi::um::winuser::GetClientRect(h_wnd, &mut rec) };
    rec
}

pub fn move_window(
    h_wnd: winapi::shared::windef::HWND,
    rect: &winapi::shared::windef::RECT,
) -> winapi::shared::minwindef::BOOL {
    unsafe {
        winapi::um::winuser::MoveWindow(
            h_wnd,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            winapi::shared::minwindef::TRUE,
        )
    }
}

pub fn is_window_visible(h_wnd: winapi::shared::windef::HWND) -> winapi::shared::minwindef::BOOL {
    unsafe { winapi::um::winuser::IsWindowVisible(h_wnd) }
}

pub fn show_window(
    h_wnd: winapi::shared::windef::HWND,
    n_cmd_show: winapi::ctypes::c_int,
) -> winapi::shared::minwindef::BOOL {
    unsafe { winapi::um::winuser::ShowWindow(h_wnd, n_cmd_show) }
}

pub fn toggle_window_visible(
    h_wnd: winapi::shared::windef::HWND,
) -> winapi::shared::minwindef::BOOL {
    use winapi::um::winuser::{SW_HIDE, SW_SHOW};
    if 1 == is_window_visible(h_wnd) {
        show_window(h_wnd, SW_HIDE)
    } else {
        show_window(h_wnd, SW_SHOW)
    }
}

// rect utilities
pub trait RectUtil
where
    Self: Sized,
{
    fn inset(&mut self, i32);
    fn shift_down(&mut self, delta: i32);
    fn shift_right(&mut self, delta: i32);
    fn split_columns(&self, at: i32) -> (Self, Self);
    fn split_rows(&self, at: i32) -> (Self, Self);
}

impl RectUtil for winapi::shared::windef::RECT {
    fn inset(&mut self, delta: i32) {
        self.left += delta;
        self.top += delta;
        self.right -= 2 * delta;
        self.bottom -= 2 * delta;
    }
    fn shift_down(&mut self, delta: i32) {
        self.top += delta;
        self.bottom -= delta;
    }
    fn shift_right(&mut self, delta: i32) {
        self.left += delta;
        self.right -= delta;
    }
    fn split_columns(&self, at: i32) -> (Self, Self) {
        let mut l = *self;
        let mut r = *self;
        l.right = at;
        r.shift_right(at);
        (l, r)
    }
    fn split_rows(&self, at: i32) -> (Self, Self) {
        let mut u = *self;
        let mut b = *self;
        u.bottom = at;
        b.shift_down(at);
        (u, b)
    }
}

// window's proc related function

#[cfg(target_arch = "x86_64")]
pub fn get_window_wrapper<'a, T>(h_wnd: winapi::shared::windef::HWND) -> Option<&'a mut T> {
    let ptr = unsafe {
        winapi::um::winuser::GetWindowLongPtrW(h_wnd, winapi::um::winuser::GWLP_USERDATA)
    };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86")]
pub fn get_window_wrapper<'a, T>(h_wnd: winapi::shared::windef::HWND) -> Option<&'a mut T> {
    let ptr = unsafe {
        winapi::um::winuser::GetWindowLongW(h_wnd, winapi::um::winuser::GWLP_USERDATA)
    };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86_64")]
pub fn set_window_wrapper(
    h_wnd: winapi::shared::windef::HWND,
    l_param: winapi::shared::minwindef::LPARAM,
) {
    let data = unsafe { &mut *(l_param as *mut winapi::um::winuser::CREATESTRUCTW) };
    unsafe {
        winapi::um::winuser::SetWindowLongPtrW(
            h_wnd,
            winapi::um::winuser::GWLP_USERDATA,
            data.lpCreateParams as winapi::shared::basetsd::LONG_PTR,
        );
    }
}

#[cfg(target_arch = "x86")]
pub fn set_window_wrapper(
    h_wnd: winapi::shared::windef::HWND,
    l_param: winapi::shared::minwindef::LPARAM,
) {
    let data = unsafe { &mut *(l_param as *mut winapi::um::winuser::CREATESTRUCTW) };
    unsafe {
        winapi::um::winuser::SetWindowLongW(
            h_wnd,
            winapi::um::winuser::GWLP_USERDATA,
            data.lpCreateParams as winapi::um::winnt::LONG,
        );
    }
}

pub trait Windowed {
    fn window_proc(
        &mut self,
        msg: winapi::shared::minwindef::UINT,
        w_param: winapi::shared::minwindef::WPARAM,
        l_param: winapi::shared::minwindef::LPARAM,
    ) -> Option<winapi::shared::minwindef::LRESULT>;
}

pub unsafe extern "system" fn window_proc_generic<T: Windowed>(
    h_wnd: winapi::shared::windef::HWND,
    msg: winapi::shared::minwindef::UINT,
    w_param: winapi::shared::minwindef::WPARAM,
    l_param: winapi::shared::minwindef::LPARAM,
) -> winapi::shared::minwindef::LRESULT {
    if msg == winapi::um::winuser::WM_CREATE {
        set_window_wrapper(h_wnd, l_param);
    }
    // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param);
    if let Some(this) = get_window_wrapper::<T>(h_wnd) {
        if let Some(out) = this.window_proc(msg, w_param, l_param) {
            return out;
        }
    }
    winapi::um::winuser::DefWindowProcW(h_wnd, msg, w_param, l_param)
}
