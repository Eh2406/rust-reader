use winapi;
use ole32;

use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::fmt::Display;

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

impl<T> ToWide for T where T: AsRef<OsStr> {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}

pub struct Com {
    hr: winapi::HRESULT,
}

impl Com {
    pub fn new() -> Com {
        println!("new for Com");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms678543.aspx
        let hr = unsafe { ole32::CoInitialize(ptr::null_mut()) };
        if failed(hr) {
            panic!("failed for Com");
        }
        Com { hr: hr }
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms688715.aspx
        if self.hr != winapi::RPC_E_CHANGED_MODE {
            unsafe {
                ole32::CoUninitialize();
            }
        }
        println!("drop for Com");
    }
}

pub struct SpVoice<'a> {
    // https://msdn.microsoft.com/en-us/library/ms723602.aspx
    voice: &'a mut winapi::ISpVoice,
}

#[allow(dead_code)]
impl<'a> SpVoice<'a> {
    pub fn new() -> SpVoice<'a> {
        println!("new for SpVoice");
        let mut hr;
        let mut voice: *mut winapi::ISpVoice = ptr::null_mut();
        let sp_voice = "SAPI.SpVoice".to_wide_null();
        let mut clsid_spvoice = winapi::CLSID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        };

        unsafe {
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
            SpVoice { voice: &mut *voice }
        }
    }

    pub fn speak<T: ToWide + Display>(&mut self, string: T) {
        unsafe {
            println!("speaking: {:}", string);
            self.voice.Speak(string.to_wide_null().as_ptr(), 19, ptr::null_mut());
        }
    }

    pub fn wait(&mut self) {
        unsafe {
            self.voice.WaitUntilDone(winapi::INFINITE);
        }
    }

    pub fn speak_wait<T: ToWide + Display>(&mut self, string: T) {
        self.speak(string);
        self.wait();
    }

    pub fn pause(&mut self) {
        unsafe {
            self.voice.Pause();
        }
    }

    pub fn resume(&mut self) {
        unsafe {
            self.voice.Resume();
        }
    }

    pub fn set_rate(&mut self, rate: i32) {
        unsafe {
            self.voice.SetRate(rate);
        }
    }

    pub fn get_rate(&mut self) -> i32 {
        let mut rate = 0;
        unsafe {
            self.voice.GetRate(&mut rate);
        }
        rate
    }

    pub fn set_volume(&mut self, volume: u16) {
        unsafe {
            self.voice.SetVolume(volume);
        }
    }

    pub fn get_volume(&mut self) -> u16 {
        let mut volume = 0;
        unsafe {
            self.voice.GetVolume(&mut volume);
        }
        volume
    }

    pub fn get_status(&mut self) -> winapi::SPVOICESTATUS {
        let mut status = winapi::SPVOICESTATUS {
            ulCurrentStream: 0,
            ulLastStreamQueued: 0,
            hrLastResult: 0,
            dwRunningState: 0,
            ulInputWordPos: 0,
            ulInputWordLen: 0,
            ulInputSentPos: 0,
            ulInputSentLen: 0,
            lBookmarkId: 0,
            PhonemeId: 0,
            VisemeId: winapi::SP_VISEME_0,
            dwReserved1: 0,
            dwReserved2: 0,
        };
        unsafe {
            self.voice.GetStatus(&mut status, 0u16 as *mut *mut u16);
        }
        status
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
