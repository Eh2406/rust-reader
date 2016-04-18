use winapi;
use user32;
use kernel32;

use std::ops::Range;
use std::mem;

pub use wide_string::*;

// waiting for winapi
pub mod winapi_stub {
    #![allow(dead_code, non_snake_case)]
    use winapi::HMENU;

    pub const ID_EDITCHILD: HMENU = 100 as HMENU;
}

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

pub fn get_message() -> Option<winapi::MSG> {
    use std::ptr::null_mut;
    let mut msg: winapi::MSG = unsafe { mem::zeroed() };
    if unsafe { user32::GetMessageW(&mut msg, null_mut(), 0, 0) } <= 0 {
        return None;
    }
    Some(msg)
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

pub fn set_edit_selection(h_wnd: winapi::HWND, celec: Range<usize>) -> winapi::LRESULT {
    unsafe {
        user32::SendMessageW(h_wnd,
                             winapi::EM_SETSEL as winapi::UINT,
                             celec.start as winapi::WPARAM,
                             celec.end as winapi::LPARAM)
    }
}

pub fn set_edit_scroll_caret(h_wnd: winapi::HWND) -> winapi::LRESULT {
    unsafe {
        user32::SendMessageW(h_wnd,
                             winapi::EM_SCROLLCARET as winapi::UINT,
                             0 as winapi::WPARAM,
                             0 as winapi::LPARAM)
    }
}

pub fn get_client_rect(h_wnd: winapi::HWND) -> winapi::RECT {
    let mut rec: winapi::RECT = unsafe { mem::zeroed() };
    unsafe { user32::GetClientRect(h_wnd, &mut rec) };
    rec
}

pub fn move_window(h_wnd: winapi::HWND, rect: &winapi::RECT) -> winapi::BOOL {
    unsafe {
        user32::MoveWindow(h_wnd,
                           rect.left,
                           rect.top,
                           rect.right,
                           rect.bottom,
                           winapi::TRUE)
    }
}

pub fn is_window_visible(h_wnd: winapi::HWND) -> winapi::BOOL {
    unsafe { user32::IsWindowVisible(h_wnd) }
}

pub fn show_window(hWnd: winapi::HWND, nCmdShow: winapi::c_int) -> winapi::BOOL {
    unsafe { user32::ShowWindow(hWnd, nCmdShow) }
}

pub fn toggle_window_visible(h_wnd: winapi::HWND) -> winapi::BOOL {
    if 1 == is_window_visible(h_wnd) {
        show_window(h_wnd, winapi::SW_HIDE)
    } else {
        show_window(h_wnd, winapi::SW_SHOW)
    }
}

// rect utilities
pub trait RectUtil {
    fn inset(&mut self, i32);
}

impl RectUtil for winapi::RECT {
    fn inset(&mut self, delta: i32) {
        self.left += delta;
        self.top += delta;
        self.right -= 2 * delta;
        self.bottom -= 2 * delta;
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
        user32::SetWindowLongPtrW(h_wnd,
                                  winapi::GWLP_USERDATA,
                                  data.lpCreateParams as winapi::LONG_PTR);
    }
}

#[cfg(target_arch = "x86")]
pub fn set_window_wrapper(h_wnd: winapi::HWND, l_param: winapi::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winapi::CREATESTRUCTW) };
    unsafe {
        user32::SetWindowLongW(h_wnd,
                               winapi::GWLP_USERDATA,
                               data.lpCreateParams as winapi::LONG);
    }
}

pub trait Windowed {
    fn window_proc(&mut self,
                   msg: winapi::UINT,
                   w_param: winapi::WPARAM,
                   l_param: winapi::LPARAM)
                   -> Option<winapi::LRESULT>;
}

pub unsafe extern "system" fn window_proc_generic<T: Windowed>(h_wnd: winapi::HWND,
                                                               msg: winapi::UINT,
                                                               w_param: winapi::WPARAM,
                                                               l_param: winapi::LPARAM)
                                                               -> winapi::LRESULT {
    if msg == winapi::WM_CREATE {
        set_window_wrapper(h_wnd, l_param);
    }
    // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param);
    if let Some(this) = get_window_wrapper::<T>(h_wnd) {
        if let Some(out) = this.window_proc(msg, w_param, l_param) {
            return out;
        }
    }
    return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
}
