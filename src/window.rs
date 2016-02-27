use winapi;
use user32;
use kernel32;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ops::Range;
use std::mem;

// waiting for winapi
pub mod winapi_stub {
    #![allow(dead_code)]
    use winapi::{DWORD, HMENU};
    pub const ES_AUTOHSCROLL: DWORD = 128;
    pub const ES_AUTOVSCROLL: DWORD = 64;
    pub const ES_CENTER: DWORD = 1;
    pub const ES_LEFT: DWORD = 0;
    pub const ES_LOWERCASE: DWORD = 16;
    pub const ES_MULTILINE: DWORD = 4;
    pub const ES_NOHIDESEL: DWORD = 256;
    pub const ES_NUMBER: DWORD = 0x2000;
    pub const ES_OEMCONVERT: DWORD = 0x400;
    pub const ES_PASSWORD: DWORD = 32;
    pub const ES_READONLY: DWORD = 0x800;
    pub const ES_RIGHT: DWORD = 2;
    pub const ES_UPPERCASE: DWORD = 8;
    pub const ES_WANTRETURN: DWORD = 4096;

    pub const EM_CANUNDO: DWORD = 198;
    pub const EM_CHARFROMPOS: DWORD = 215;
    pub const EM_EMPTYUNDOBUFFER: DWORD = 205;
    pub const EM_FMTLINES: DWORD = 200;
    pub const EM_GETFIRSTVISIBLELINE: DWORD = 206;
    pub const EM_GETHANDLE: DWORD = 189;
    pub const EM_GETLIMITTEXT: DWORD = 213;
    pub const EM_GETLINE: DWORD = 196;
    pub const EM_GETLINECOUNT: DWORD = 186;
    pub const EM_GETMARGINS: DWORD = 212;
    pub const EM_GETMODIFY: DWORD = 184;
    pub const EM_GETPASSWORDCHAR: DWORD = 210;
    pub const EM_GETRECT: DWORD = 178;
    pub const EM_GETSEL: DWORD = 176;
    pub const EM_GETTHUMB: DWORD = 190;
    pub const EM_GETWORDBREAKPROC: DWORD = 209;
    pub const EM_LIMITTEXT: DWORD = 197;
    pub const EM_LINEFROMCHAR: DWORD = 201;
    pub const EM_LINEINDEX: DWORD = 187;
    pub const EM_LINELENGTH: DWORD = 193;
    pub const EM_LINESCROLL: DWORD = 182;
    pub const EM_POSFROMCHAR: DWORD = 214;
    pub const EM_REPLACESEL: DWORD = 194;
    pub const EM_SCROLL: DWORD = 181;
    pub const EM_SCROLLCARET: DWORD = 183;
    pub const EM_SETHANDLE: DWORD = 188;
    pub const EM_SETLIMITTEXT: DWORD = 197;
    pub const EM_SETMARGINS: DWORD = 211;
    pub const EM_SETMODIFY: DWORD = 185;
    pub const EM_SETPASSWORDCHAR: DWORD = 204;
    pub const EM_SETREADONLY: DWORD = 207;
    pub const EM_SETRECT: DWORD = 179;
    pub const EM_SETRECTNP: DWORD = 180;
    pub const EM_SETSEL: DWORD = 177;
    pub const EM_SETTABSTOPS: DWORD = 203;
    pub const EM_SETWORDBREAKPROC: DWORD = 208;
    pub const EM_UNDO: DWORD = 199;

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

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T: AsRef<OsStr>> ToWide for T {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
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
                             winapi_stub::EM_SETSEL,
                             celec.start as winapi::WPARAM,
                             celec.end as winapi::LPARAM)
    }
}

pub fn set_edit_scroll_caret(h_wnd: winapi::HWND) -> winapi::LRESULT {
    unsafe {
        user32::SendMessageW(h_wnd,
                             winapi_stub::EM_SCROLLCARET,
                             0 as winapi::WPARAM,
                             0 as winapi::LPARAM)
    }
}

pub fn get_client_rect(h_wnd: winapi::HWND) -> winapi::RECT {
    let mut rec: winapi::RECT = unsafe { mem::zeroed() };
    unsafe { user32::GetClientRect(h_wnd, &mut rec) };
    rec
}

// window's proc related function

pub fn get_window_wrapper<'a, T>(h_wnd: winapi::HWND) -> Option<&'a mut T> {
    let ptr: winapi::LONG_PTR = unsafe { user32::GetWindowLongPtrW(h_wnd, winapi::GWLP_USERDATA) };
    if ptr > 0 {
        Some(unsafe { &mut *(ptr as *mut T) })
    } else {
        None
    }
}

pub fn set_window_wrapper(h_wnd: winapi::HWND, l_param: winapi::LPARAM) {
    let data = unsafe { &mut *(l_param as *mut winapi::CREATESTRUCTW) };
    unsafe {
        user32::SetWindowLongPtrW(h_wnd,
                                  winapi::GWLP_USERDATA,
                                  data.lpCreateParams as winapi::LONG_PTR);
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
    match msg {
        winapi::WM_NCCREATE => set_window_wrapper(h_wnd, l_param),
        _ => {
            // println!("sinproc: msg:{:?} w_param:{:?} l_param:{:?}", msg, w_param, l_param)
            if let Some(this) = get_window_wrapper::<T>(h_wnd) {
                if let Some(out) = this.window_proc(msg, w_param, l_param) {
                    return out;
                }
            }
        }

    }
    return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
}
