extern crate winapi;
extern crate ole32;
extern crate clipboard_win;

use clipboard_win::{get_clipboard_string};
use std::mem;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[inline]
fn failed(hr: winapi::HRESULT) -> bool {
    hr < 0
}

#[inline]
fn succeeded(hr: winapi::HRESULT) -> bool {
    !failed(hr)
}

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T> ToWide for T where T: AsRef<OsStr> {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}

struct Com {
    hr: winapi::HRESULT,
}

impl Com {
    fn new() -> Com {
        println!("new for Con");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms678543(v=vs.85).aspx
        let hr = unsafe {ole32::CoInitialize(ptr::null_mut())};
        if failed(hr) {
            panic!("failed for Con");
        }
        Com {hr: hr}
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms688715(v=vs.85).aspx
        if self.hr != winapi::RPC_E_CHANGED_MODE {
            unsafe {
                ole32::CoUninitialize();
            }
        }
        println!("drop for Con");
    }
}

struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602(VS.85).aspx
    voice: &'a mut winapi::ISpVoice,
}

impl<'a> SpVoice<'a> {
    fn new() -> SpVoice<'a> {
        println!("new for SpVoice");
        let mut hr;
        let mut voice: *mut winapi::ISpVoice = ptr::null_mut();
        let sp_voice = "SAPI.SpVoice".to_wide_null();

        unsafe {
            let mut clsid_spvoice: winapi::CLSID = mem::uninitialized();

            hr = ole32::CLSIDFromProgID(&sp_voice[0], &mut clsid_spvoice);
            if failed(hr) {
                panic!("failed for SpVoice at CLSIDFromProgID");
            }

            hr = ole32::CoCreateInstance(
                &clsid_spvoice,
                ptr::null_mut(),
                winapi::CLSCTX_ALL,
                &winapi::UuidOfISpVoice,
                &mut voice as *mut *mut winapi::ISpVoice as *mut *mut winapi::c_void
            );
            if failed(hr) {
                panic!("failed for SpVoice at CoCreateInstance");
            }
            SpVoice {
                voice: &mut *voice,
            }
        }
    }

    fn speak (&mut self, string: &str) {
        unsafe {
            println!("befor speak");
            self.voice.Speak(string.to_wide_null().as_ptr(), 19, ptr::null_mut());
            println!("after speak");
        }
    }

    fn wait (&mut self) {
        unsafe {
            self.voice.WaitUntilDone(winapi::INFINITE);
        }
    }

    fn speak_wait (&mut self, string: &str) {
        self.speak(string);
        self.wait();
    }

}

impl<'a> Drop for SpVoice<'a> {
    fn drop(&mut self) {
        unsafe {
            self.voice.Release();
        }
        println!("drop for SpVoice");
    }
}

fn main() {
    let com = Com::new();
    let mut voice = SpVoice::new();

    voice.speak_wait("Converting format back and forth.");
    voice.speak_wait("You have in your clipboard.");
    match get_clipboard_string() {
        Ok(x) => voice.speak_wait(&x),
        Err(x) => {
            voice.speak_wait("oops... error.");
            println!("{:?}", x);
        }
    }
    voice.speak_wait("Done.");
}
