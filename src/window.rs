use winapi;
use user32;
use kernel32;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ops;
use std::mem;
use std::ptr;

// waiting for winapi
pub mod winapi_stub {
    #![allow(dead_code)]
    use winapi;
    pub const ES_AUTOHSCROLL: winapi::DWORD = 128;
    pub const ES_AUTOVSCROLL: winapi::DWORD = 64;
    pub const ES_CENTER: winapi::DWORD = 1;
    pub const ES_LEFT: winapi::DWORD = 0;
    pub const ES_LOWERCASE: winapi::DWORD = 16;
    pub const ES_MULTILINE: winapi::DWORD = 4;
    pub const ES_NOHIDESEL: winapi::DWORD = 256;
    pub const ES_NUMBER: winapi::DWORD = 0x2000;
    pub const ES_OEMCONVERT: winapi::DWORD = 0x400;
    pub const ES_PASSWORD: winapi::DWORD = 32;
    pub const ES_READONLY: winapi::DWORD = 0x800;
    pub const ES_RIGHT: winapi::DWORD = 2;
    pub const ES_UPPERCASE: winapi::DWORD = 8;
    pub const ES_WANTRETURN: winapi::DWORD = 4096;

    pub const EM_CANUNDO: winapi::DWORD = 198;
    pub const EM_CHARFROMPOS: winapi::DWORD = 215;
    pub const EM_EMPTYUNDOBUFFER: winapi::DWORD = 205;
    pub const EM_FMTLINES: winapi::DWORD = 200;
    pub const EM_GETFIRSTVISIBLELINE: winapi::DWORD = 206;
    pub const EM_GETHANDLE: winapi::DWORD = 189;
    pub const EM_GETLIMITTEXT: winapi::DWORD = 213;
    pub const EM_GETLINE: winapi::DWORD = 196;
    pub const EM_GETLINECOUNT: winapi::DWORD = 186;
    pub const EM_GETMARGINS: winapi::DWORD = 212;
    pub const EM_GETMODIFY: winapi::DWORD = 184;
    pub const EM_GETPASSWORDCHAR: winapi::DWORD = 210;
    pub const EM_GETRECT: winapi::DWORD = 178;
    pub const EM_GETSEL: winapi::DWORD = 176;
    pub const EM_GETTHUMB: winapi::DWORD = 190;
    pub const EM_GETWORDBREAKPROC: winapi::DWORD = 209;
    pub const EM_LIMITTEXT: winapi::DWORD = 197;
    pub const EM_LINEFROMCHAR: winapi::DWORD = 201;
    pub const EM_LINEINDEX: winapi::DWORD = 187;
    pub const EM_LINELENGTH: winapi::DWORD = 193;
    pub const EM_LINESCROLL: winapi::DWORD = 182;
    pub const EM_POSFROMCHAR: winapi::DWORD = 214;
    pub const EM_REPLACESEL: winapi::DWORD = 194;
    pub const EM_SCROLL: winapi::DWORD = 181;
    pub const EM_SCROLLCARET: winapi::DWORD = 183;
    pub const EM_SETHANDLE: winapi::DWORD = 188;
    pub const EM_SETLIMITTEXT: winapi::DWORD = 197;
    pub const EM_SETMARGINS: winapi::DWORD = 211;
    pub const EM_SETMODIFY: winapi::DWORD = 185;
    pub const EM_SETPASSWORDCHAR: winapi::DWORD = 204;
    pub const EM_SETREADONLY: winapi::DWORD = 207;
    pub const EM_SETRECT: winapi::DWORD = 179;
    pub const EM_SETRECTNP: winapi::DWORD = 180;
    pub const EM_SETSEL: winapi::DWORD = 177;
    pub const EM_SETTABSTOPS: winapi::DWORD = 203;
    pub const EM_SETWORDBREAKPROC: winapi::DWORD = 208;
    pub const EM_UNDO: winapi::DWORD = 199;

    pub const ID_EDITCHILD: winapi::HMENU = 100 as winapi::HMENU;
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
    let mut msg: winapi::MSG = unsafe { mem::zeroed() };
    if unsafe { user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } <= 0 {
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

pub fn set_edit_selection(h_wnd: winapi::HWND, celec: ops::Range<usize>) -> winapi::LRESULT {
    unsafe {
        user32::SendMessageW(h_wnd,
                             winapi_stub::EM_SETSEL,
                             celec.start as winapi::WPARAM,
                             celec.end as winapi::LPARAM)
    }
}

pub fn get_client_rect(h_wnd: winapi::HWND) -> winapi::RECT {
    let mut rec: winapi::RECT = unsafe { mem::zeroed() };
    unsafe { user32::GetClientRect(h_wnd, &mut rec) };
    rec
}
