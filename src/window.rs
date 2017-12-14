use winapi;
use user32;
use kernel32;

use std::ops::Range;
use std::mem;

pub use wide_string::*;

// waiting for winapi
pub mod winapi_stub {
    #![allow(dead_code, non_snake_case)]
    use winapi::{DWORD, HMENU};

    // Static Control Constants
    //
    pub const SS_LEFT: DWORD = 0x0000_0000;
    pub const SS_CENTER: DWORD = 0x000_0001;
    pub const SS_RIGHT: DWORD = 0x0000_0002;
    pub const SS_ICON: DWORD = 0x0000_0003;
    pub const SS_BLACKRECT: DWORD = 0x0000_0004;
    pub const SS_GRAYRECT: DWORD = 0x0000_0005;
    pub const SS_WHITERECT: DWORD = 0x0000_0006;
    pub const SS_BLACKFRAME: DWORD = 0x0000_0007;
    pub const SS_GRAYFRAME: DWORD = 0x0000_0008;
    pub const SS_WHITEFRAME: DWORD = 0x0000_0009;
    pub const SS_USERITEM: DWORD = 0x0000_000A;
    pub const SS_SIMPLE: DWORD = 0x0000_000B;
    pub const SS_LEFTNOWORDWRAP: DWORD = 0x0000_000C;
    pub const SS_OWNERDRAW: DWORD = 0x0000_000D;
    pub const SS_BITMAP: DWORD = 0x0000_000E;
    pub const SS_ENHMETAFILE: DWORD = 0x0000_000F;
    pub const SS_ETCHEDHORZ: DWORD = 0x0000_0010;
    pub const SS_ETCHEDVERT: DWORD = 0x0000_0011;
    pub const SS_ETCHEDFRAME: DWORD = 0x0000_0012;
    pub const SS_TYPEMASK: DWORD = 0x0000_001F;
    pub const SS_REALSIZECONTROL: DWORD = 0x0000_0040;
    pub const SS_NOPREFIX: DWORD = 0x0000_0080;
    pub const SS_NOTIFY: DWORD = 0x0000_0100;
    pub const SS_CENTERIMAGE: DWORD = 0x0000_0200;
    pub const SS_RIGHTJUST: DWORD = 0x0000_0400;
    pub const SS_REALSIZEIMAGE: DWORD = 0x0000_0800;
    pub const SS_SUNKEN: DWORD = 0x0000_1000;
    pub const SS_EDITCONTROL: DWORD = 0x0000_2000;
    pub const SS_ENDELLIPSIS: DWORD = 0x0000_4000;
    pub const SS_PATHELLIPSIS: DWORD = 0x0000_8000;
    pub const SS_WORDELLIPSIS: DWORD = 0x0000_C000;
    pub const SS_ELLIPSISMASK: DWORD = 0x0000_C000;

    pub const ID_EDITCHILD: HMENU = 100 as HMENU;
}

#[inline]
pub fn failed(hr: winapi::HRESULT) -> bool {
    hr < 0
}

#[inline]
#[allow(dead_code)]
pub fn succeeded(hr: winapi::HRESULT) -> bool {
    !failed(hr)
}

pub fn get_message() -> Option<winapi::MSG> {
    use std::ptr::null_mut;
    let mut msg: winapi::MSG = unsafe { mem::zeroed() };
    if unsafe { user32::GetMessageW(&mut msg, null_mut(), 0, 0) } <= 0 {
        return None;
    }
    Some(msg)
}

pub fn set_console_title(title: &WideString) -> i32 {
    unsafe { kernel32::SetConsoleTitleW(title.as_ptr()) }
}

pub fn set_window_text(h_wnd: winapi::HWND, wide: &WideString) -> winapi::BOOL {
    unsafe { user32::SetWindowTextW(h_wnd, wide.as_ptr()) }
}

pub fn close() {
    unsafe { user32::PostQuitMessage(0) }
}

pub fn set_edit_selection(h_wnd: winapi::HWND, celec: &Range<usize>) -> winapi::LRESULT {
    unsafe {
        user32::SendMessageW(
            h_wnd,
            winapi::UINT::from(winapi::EM_SETSEL),
            celec.start as winapi::WPARAM,
            celec.end as winapi::LPARAM,
        )
    }
}

pub fn set_edit_scroll_caret(h_wnd: winapi::HWND) -> winapi::LRESULT {
    unsafe {
        user32::SendMessageW(
            h_wnd,
            winapi::UINT::from(winapi::EM_SCROLLCARET),
            0 as winapi::WPARAM,
            0 as winapi::LPARAM,
        )
    }
}

pub fn get_client_rect(h_wnd: winapi::HWND) -> winapi::RECT {
    let mut rec: winapi::RECT = unsafe { mem::zeroed() };
    unsafe { user32::GetClientRect(h_wnd, &mut rec) };
    rec
}

pub fn move_window(h_wnd: winapi::HWND, rect: &winapi::RECT) -> winapi::BOOL {
    unsafe {
        user32::MoveWindow(
            h_wnd,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            winapi::TRUE,
        )
    }
}

pub fn is_window_visible(h_wnd: winapi::HWND) -> winapi::BOOL {
    unsafe { user32::IsWindowVisible(h_wnd) }
}

pub fn show_window(h_wnd: winapi::HWND, n_cmd_show: winapi::c_int) -> winapi::BOOL {
    unsafe { user32::ShowWindow(h_wnd, n_cmd_show) }
}

pub fn toggle_window_visible(h_wnd: winapi::HWND) -> winapi::BOOL {
    if 1 == is_window_visible(h_wnd) {
        show_window(h_wnd, winapi::SW_HIDE)
    } else {
        show_window(h_wnd, winapi::SW_SHOW)
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

impl RectUtil for winapi::RECT {
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
pub fn get_window_wrapper<'a, T>(h_wnd: winapi::HWND) -> Option<&'a mut T> {
    let ptr = unsafe { user32::GetWindowLongPtrW(h_wnd, winapi::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86")]
pub fn get_window_wrapper<'a, T>(h_wnd: winapi::HWND) -> Option<&'a mut T> {
    let ptr = unsafe { user32::GetWindowLongW(h_wnd, winapi::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

#[cfg(target_arch = "x86_64")]
pub fn set_window_wrapper(h_wnd: winapi::HWND, l_param: winapi::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winapi::CREATESTRUCTW) };
    unsafe {
        user32::SetWindowLongPtrW(
            h_wnd,
            winapi::GWLP_USERDATA,
            data.lpCreateParams as winapi::LONG_PTR,
        );
    }
}

#[cfg(target_arch = "x86")]
pub fn set_window_wrapper(h_wnd: winapi::HWND, l_param: winapi::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winapi::CREATESTRUCTW) };
    unsafe {
        user32::SetWindowLongW(
            h_wnd,
            winapi::GWLP_USERDATA,
            data.lpCreateParams as winapi::LONG,
        );
    }
}

pub trait Windowed {
    fn window_proc(
        &mut self,
        msg: winapi::UINT,
        w_param: winapi::WPARAM,
        l_param: winapi::LPARAM,
    ) -> Option<winapi::LRESULT>;
}

pub unsafe extern "system" fn window_proc_generic<T: Windowed>(
    h_wnd: winapi::HWND,
    msg: winapi::UINT,
    w_param: winapi::WPARAM,
    l_param: winapi::LPARAM,
) -> winapi::LRESULT {
    if msg == winapi::WM_CREATE {
        set_window_wrapper(h_wnd, l_param);
    }
    // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param);
    if let Some(this) = get_window_wrapper::<T>(h_wnd) {
        if let Some(out) = this.window_proc(msg, w_param, l_param) {
            return out;
        }
    }
    user32::DefWindowProcW(h_wnd, msg, w_param, l_param)
}
