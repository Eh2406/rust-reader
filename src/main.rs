// Comment out the following line in order to see console output
//#![cfg_attr(not(test), windows_subsystem = "windows")]

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging as wm,
};

#[cfg(test)]
#[macro_use]
extern crate lazy_static;
mod wide_string;
mod window;
use crate::window::*;

mod sapi;
use crate::sapi::*;

mod clipboard;
use crate::clipboard::*;

mod actions;
use crate::actions::*;

mod hot_key;
use crate::hot_key::*;

mod settings;
use crate::settings::*;

mod clean_text;
use crate::clean_text::*;

struct State {
    voice: Box<SpVoice>,
    settings: Box<SettingsWindow>,
    hk: Vec<HotKey>,
}

impl State {
    fn read(&mut self) {
        self.voice.resume();
        match get_text() {
            Ok(x) => self.voice.speak(clean_text::<WideString>(
                &x,
                &self.settings.get_inner_settings().cleaners,
            )),
            Err(x) => {
                self.voice.speak("oops. error.");
                println!("{:?}", x);
            }
        }
    }

    fn reload_settings(&mut self) {
        let mut speech = String::new();
        if self.settings.get_mut_inner_settings().reload_from_file() {
            self.hk.clear();
            self.hk = setup_hotkeys(self.settings.get_mut_inner_settings());
            self.settings.get_mut_inner_settings().rate =
                self.voice.set_rate(self.settings.get_inner_settings().rate);
            self.voice.set_voice_by_name(self.settings.get_mut_inner_settings().voice.clone());
            self.settings.get_mut_inner_settings().voice = self.voice.get_voice_name(None);
            self.settings.get_mut_inner_settings().available_voices =
                self.voice.available_voice_names();
            self.voice
                .set_time_estimater(self.settings.get_inner_settings().time_estimater.clone());
            self.settings.inner_to_file();
            speech += "reloaded settings.\r\n";
        } else {
            speech += "failed to reload settings.\r\n";
        }
        speech += &make_speech(self.settings.get_inner_settings(), &self.hk);
        self.voice.resume();
        self.voice.speak(speech);
    }

    fn show_settings(&mut self) {
        self.settings.get_mut_inner_settings().available_voices =
            self.voice.available_voice_names();
        self.settings.get_mut_inner_settings().time_estimater = self.voice.get_time_estimater();
        self.settings.inner_to_file();
        self.settings.show_window();
    }

    fn toggle_window_visible(&mut self) {
        self.voice.toggle_window_visible();
    }

    fn play_pause(&mut self) {
        match self.voice.get_status().dwRunningState {
            2 => self.voice.pause(),
            _ => self.voice.resume(),
        }
    }

    fn rate_change(&mut self, val: i32) {
        self.settings.get_mut_inner_settings().rate = self.voice.change_rate(val);
        self.settings.get_mut_inner_settings().time_estimater = self.voice.get_time_estimater();
        self.settings.inner_to_file();
        println!("rate: {:?}", self.settings.get_inner_settings().rate);
    }

    fn match_hotkey_id(&mut self, act: Action) {
        use crate::Action::*;
        match act {
            Read => self.read(),
            Close => close(),
            ReloadSettings => self.reload_settings(),
            ShowSettings => self.show_settings(),
            ToggleWindowVisible => self.toggle_window_visible(),
            PlayPause => self.play_pause(),
            RateDown => self.rate_change(-1),
            RateUp => self.rate_change(1),
        }
    }
}

fn setup_hotkeys(settings: &mut Settings) -> Vec<HotKey> {
    assert_eq!(ACTION_LIST.len(), settings.hotkeys.len());
    ACTION_LIST
        .iter()
        .zip(settings.hotkeys.iter())
        .map(|(&act, &(modifiers, vk))| {
            HotKey::new(modifiers, vk, act as i32).unwrap() // make HotKey
        })
        .collect()
}

fn press_hotkey(id: Action) {
    unsafe {
        wm::PostThreadMessageW(
            GetCurrentThreadId(),
            wm::WM_HOTKEY,
            WPARAM(id as usize),
            LPARAM(0),
        )
    };
}

fn make_speech(settings: &Settings, hk: &[HotKey]) -> String {
    let mut out = "Reading from settings at:".to_string();
    out += "\r\n";
    out += &settings.get_dir().to_string_lossy();
    out += "\r\n";
    out += "speech rate of: ";
    out += &settings.rate.to_string();
    out += "\r\n";
    out += "voice: ";
    out += &settings.voice;
    out += "\r\n";
    out += "hotkeys\r\n";
    for (act, h) in ACTION_LIST.iter().zip(hk.iter()) {
        out += &format!("{}: {}\r\n", act, h);
    }
    out += "Ready!";
    out
}

fn main() {
    let com = Com::new();
    let mut voice = SpVoice::new(&com);
    let mut settings = Settings::from_file();
    voice.set_rate(settings.rate);
    voice.set_voice_by_name(settings.voice.clone());
    //voice.set_voice_by_name("Microsoft Zira Desktop".to_string());
    voice.set_time_estimater(settings.time_estimater.clone());
    let hk = setup_hotkeys(&mut settings);
    clipboard_setup();

    let mut state = State {
        voice,
        settings: SettingsWindow::new(settings),
        hk,
    };

    state
        .voice
        .speak(make_speech(state.settings.get_inner_settings(), &state.hk));

    while let Some(msg) = get_message() {
        match msg.message {
            wm::WM_HOTKEY if msg.wParam.0 < state.hk.len() => {
                state.match_hotkey_id(ACTION_LIST[msg.wParam.0])
            }
            _ => {
                // println!("{:?}", msg);
                unsafe {
                    wm::TranslateMessage(&msg);
                    wm::DispatchMessageW(&msg);
                }
            }
        }
    }
    state.voice.resume();
    state.voice.speak_wait("bye!");
    state.settings.get_mut_inner_settings().time_estimater = state.voice.get_time_estimater();
    state.settings.inner_to_file();
}
