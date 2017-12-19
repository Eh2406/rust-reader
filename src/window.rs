use winapi;
use winapi::um::winnt;
use winapi::um::winuser;
use winapi::shared::minwindef;
use winapi::shared::windef;

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
pub fn failed(hr: winnt::HRESULT) -> bool {
    hr < 0
}

#[inline]
#[allow(dead_code)]
pub fn succeeded(hr: winnt::HRESULT) -> bool {
    !failed(hr)
}

pub fn get_message() -> Option<winuser::MSG> {
    use std::ptr::null_mut;
    let mut msg: winuser::MSG = unsafe { mem::zeroed() };
    if unsafe { winuser::GetMessageW(&mut msg, null_mut(), 0, 0) } <= 0 {
        return None;
    }
    Some(msg)
}

pub fn set_console_title(title: &WideString) -> i32 {
    unsafe { winapi::um::wincon::SetConsoleTitleW(title.as_ptr()) }
}

pub fn set_window_text(h_wnd: windef::HWND, wide: &WideString) -> minwindef::BOOL {
    unsafe { winuser::SetWindowTextW(h_wnd, wide.as_ptr()) }
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
    unsafe {
        winuser::SendMessageW(
            h_wnd,
            minwindef::UINT::from(winuser::EM_SCROLLCARET),
            0 as minwindef::WPARAM,
            0 as minwindef::LPARAM,
        )
    }
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
    fn inset(&mut self, i32);
    fn shift_down(&mut self, delta: i32);
    fn shift_right(&mut self, delta: i32);
    fn split_columns(&self, at: i32) -> (Self, Self);
    fn split_rows(&self, at: i32) -> (Self, Self);
}

impl RectUtil for windef::RECT {
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
