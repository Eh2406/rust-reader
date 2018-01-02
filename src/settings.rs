use preferences::{prefs_base_dir, AppInfo, Preferences};
use winapi::um::winuser::{VK_OEM_2, VK_ESCAPE, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS};
use winapi::shared::windef;
use winapi::um::winuser;
use winapi::um::commctrl;
use winapi::shared::minwindef;
use clean_text::RegexCleanerPair;
use average::Variance;
use std::ptr::null_mut;
use wide_string::WideString;
use window::*;
use hot_key::*;
use itertools::Itertools;

const APP_INFO: AppInfo = AppInfo {
    name: "rust_reader",
    author: "us",
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub rate: i32,
    pub hotkeys: [(u32, u32); 8],
    pub cleaners: Vec<RegexCleanerPair>,
    #[serde(default)] pub time_estimater: [Variance; 21],
}

pub struct SettingsWindow {
    settings: Settings,
    window: windef::HWND,
    rate: (windef::HWND, windef::HWND),
    hotkeys: [(windef::HWND, windef::HWND); 8],
    cleaners: Vec<(Option<bool>, windef::HWND, windef::HWND)>,
    save: windef::HWND,
}

impl SettingsWindow {
    pub fn new(s: Settings) -> Box<SettingsWindow> {
        let mut out = Box::new(SettingsWindow {
            settings: s,
            window: null_mut(),
            rate: (null_mut(), null_mut()),
            hotkeys: [(null_mut(), null_mut()); 8],
            cleaners: Vec::new(),
            save: null_mut(),
        });

        let window_class_name: WideString = "setings_window_class_name".into();
        unsafe {
            winuser::RegisterClassW(&winuser::WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc_generic::<SettingsWindow>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: null_mut(),
                hIcon: winuser::LoadIconW(null_mut(), winuser::IDI_APPLICATION),
                hCursor: winuser::LoadCursorW(null_mut(), winuser::IDI_APPLICATION),
                hbrBackground: 16 as windef::HBRUSH,
                lpszMenuName: null_mut(),
                lpszClassName: window_class_name.as_ptr(),
            });
            out.window = winuser::CreateWindowExW(
                0,
                window_class_name.as_ptr(),
                &0u16,
                winuser::WS_OVERLAPPEDWINDOW | winuser::WS_CLIPSIBLINGS,
                0,
                0,
                0,
                0,
                winuser::GetDesktopWindow(),
                null_mut(),
                null_mut(),
                &mut *out as *mut _ as minwindef::LPVOID,
            );
            let wide_static: WideString = "STATIC".into();
            commctrl::InitCommonControls();
            let wide_trackbar: WideString = "msctls_trackbar32".into();
            out.rate.1 = winuser::CreateWindowExW(
                0,
                wide_trackbar.as_ptr(),
                &0u16,
                winuser::WS_CHILD | winuser::WS_VISIBLE | commctrl::TBS_AUTOTICKS
                    | commctrl::TBS_BOTTOM,
                0,
                0,
                0,
                0,
                out.window,
                null_mut(),
                null_mut(),
                null_mut(),
            );
            winuser::SendMessageW(
                out.rate.1,
                commctrl::TBM_SETRANGE,
                0,
                minwindef::MAKELONG(0, 20) as isize,
            );
            winuser::SendMessageW(out.rate.1, commctrl::TBM_SETPAGESIZE, 0, 1);
            out.rate.0 = winuser::CreateWindowExW(
                0,
                wide_static.as_ptr(),
                &0u16,
                winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::SS_CENTER | winuser::SS_NOPREFIX,
                0,
                0,
                0,
                0,
                out.window,
                null_mut(),
                null_mut(),
                null_mut(),
            );
            let wide_button: WideString = "BUTTON".into();
            let wide_save: WideString = "save".into();
            out.save = winuser::CreateWindowExW(
                0,
                wide_button.as_ptr(),
                wide_save.as_ptr(),
                winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::BS_CENTER
                    | winuser::BS_DEFPUSHBUTTON,
                0,
                0,
                0,
                0,
                out.window,
                null_mut(),
                null_mut(),
                null_mut(),
            );

            let window = out.window;
            let wide_hotkey_class: WideString = "msctls_hotkey32".into();

            let mut icex: commctrl::INITCOMMONCONTROLSEX = ::std::mem::zeroed();
            icex.dwSize = ::std::mem::size_of::<commctrl::INITCOMMONCONTROLSEX>() as u32;
            icex.dwICC = commctrl::ICC_HOTKEY_CLASS;
            commctrl::InitCommonControlsEx(&icex);

            for (act, ht) in ::actions::ACTION_LIST.iter().zip(out.hotkeys.iter_mut()) {
                let wide_hotkey_name: WideString = format!("{}", act).into();
                ht.0 = winuser::CreateWindowExW(
                    0,
                    wide_static.as_ptr(),
                    wide_hotkey_name.as_ptr(),
                    winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::SS_CENTER
                        | winuser::SS_NOPREFIX,
                    0,
                    0,
                    0,
                    0,
                    window,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                );
                ht.1 = winuser::CreateWindowExW(
                    0,
                    wide_hotkey_class.as_ptr(),
                    &0u16,
                    winuser::WS_CHILD | winuser::WS_VISIBLE,
                    0,
                    0,
                    0,
                    0,
                    window,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                );
                winuser::SendMessageW(
                    ht.1,
                    commctrl::HKM_SETRULES,
                    (commctrl::HKCOMB_NONE | commctrl::HKCOMB_S) as usize,
                    commctrl::HOTKEYF_ALT as isize,
                );
            }
        }
        enable_window(out.save, false);
        set_window_text(out.window, &"reader settings".into());
        out.get_inner_all();
        move_window(
            out.window,
            &windef::RECT {
                left: 0,
                top: 0,
                right: 400,
                bottom: 400,
            },
        );
        show_window(out.window, winuser::SW_SHOWNORMAL);
        out.toggle_window_visible();
        out
    }

    fn add_cleaner(&mut self) {
        let wide_edit: WideString = "EDIT".into();
        self.cleaners.push(unsafe {
            (
                None,
                winuser::CreateWindowExW(
                    winuser::WS_EX_CLIENTEDGE,
                    wide_edit.as_ptr(),
                    &0u16,
                    winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::WS_BORDER | winuser::ES_LEFT
                        | winuser::ES_NOHIDESEL,
                    0,
                    0,
                    0,
                    0,
                    self.window,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                ),
                winuser::CreateWindowExW(
                    winuser::WS_EX_CLIENTEDGE,
                    wide_edit.as_ptr(),
                    &0u16,
                    winuser::WS_CHILD | winuser::WS_VISIBLE | winuser::WS_BORDER | winuser::ES_LEFT
                        | winuser::ES_NOHIDESEL,
                    0,
                    0,
                    0,
                    0,
                    self.window,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                ),
            )
        })
    }

    pub fn get_inner_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn get_mut_inner_settings(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn toggle_window_visible(&self) -> minwindef::BOOL {
        toggle_window_visible(self.window)
    }

    pub fn show_window(&self) -> minwindef::BOOL {
        show_window(self.window, winuser::SW_SHOW)
    }

    pub fn get_inner_rate(&mut self) -> i32 {
        let rate = self.settings.rate;
        unsafe {
            winuser::SendMessageW(self.rate.1, commctrl::TBM_SETPOS, 1, (rate + 10) as isize);
        }
        set_window_text(self.rate.0, &format!("reading at rate: {}", rate).into());
        rate
    }

    pub fn get_inner_hotkeys(&self) -> [(u32, u32); 8] {
        for (&(a, b), hwnd) in self.settings.hotkeys.iter().zip(self.hotkeys.iter()) {
            unsafe {
                winuser::SendMessageW(
                    hwnd.1,
                    commctrl::HKM_SETHOTKEY,
                    minwindef::MAKEWORD(b as u8, convert_mod(a as u8)) as usize,
                    0,
                );
            }
        }
        self.settings.hotkeys
    }

    pub fn get_inner_cleaners(&mut self) -> &[RegexCleanerPair] {
        use itertools::EitherOrBoth::{Both, Left, Right};
        while self.cleaners.len() < self.settings.cleaners.len() {
            self.add_cleaner();
        }
        for mat in self.cleaners
            .iter_mut()
            .zip_longest(self.settings.cleaners.iter())
        {
            match mat {
                Both(cl, rexpar) => {
                    let (re, pal) = rexpar.to_parts();
                    cl.0 = None;
                    set_window_text(cl.1, &re.as_str().into());
                    set_window_text(cl.2, &pal.into());
                }
                Right(_) => panic!("oops"),
                Left(cl) => {
                    cl.0 = None;
                    set_window_text(cl.1, &"".into());
                    set_window_text(cl.2, &"".into());
                }
            }
        }
        &self.settings.cleaners
    }

    fn get_inner_all(&mut self) {
        self.get_inner_rate();
        self.get_inner_hotkeys();
        self.get_inner_cleaners();
    }

    pub fn inner_to_file(&mut self) {
        self.get_inner_all();
        self.settings.to_file()
    }
}

impl Windowed for SettingsWindow {
    fn window_proc(
        &mut self,
        msg: minwindef::UINT,
        w_param: minwindef::WPARAM,
        l_param: minwindef::LPARAM,
    ) -> Option<minwindef::LRESULT> {
        match msg {
            winuser::WM_CLOSE => {
                show_window(self.window, winuser::SW_HIDE);
                return Some(0);
            }
            winuser::WM_SIZE => {
                let mut rect = get_client_rect(self.window);
                if (w_param <= 2) && rect.right > 0 && rect.bottom > 0 {
                    rect.inset(3);
                    let mut rect = rect.split_rows(rect.bottom - 25);
                    move_window(self.save, &rect.1);
                    rect = rect.0.split_rows(25);
                    let (l, r) = rect.0.split_columns(160);
                    move_window(self.rate.0, &l);
                    move_window(self.rate.1, &r);
                    for &ht in &self.hotkeys {
                        rect = rect.1.split_rows(25);
                        let (l, r) = rect.0.split_columns(160);
                        move_window(ht.0, &l);
                        move_window(ht.1, &r);
                    }
                    rect.1.shift_down(5);
                    let mll = self.cleaners
                        .iter()
                        .map(|&(_, a, _)| get_window_text_length(a))
                        .max()
                        .unwrap_or(0) + 1;
                    let mlr = self.cleaners
                        .iter()
                        .map(|&(_, _, b)| get_window_text_length(b))
                        .max()
                        .unwrap_or(0) + 1;
                    let split_at = rect.1.right * mll / (mll + mlr);
                    for &ht in &self.cleaners {
                        rect = rect.1.split_rows(25);
                        let (l, r) = rect.0.split_columns(split_at);
                        move_window(ht.1, &l);
                        move_window(ht.2, &r);
                    }
                    return Some(0);
                }
            }
            winuser::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param as *mut winuser::MINMAXINFO) };
                data.ptMinTrackSize.x = 340;
                data.ptMinTrackSize.y =
                    (55 + 25 * (2 + self.hotkeys.len()) + 25 * self.cleaners.len()) as i32;
                return Some(0);
            }
            winuser::WM_COMMAND | winuser::WM_HSCROLL => {
                let mut changed = false;
                let mut invalid = false;

                let saving = self.save as isize == l_param
                    && minwindef::HIWORD(w_param as u32) == winuser::BN_CLICKED;

                // rate change
                let new_rate =
                    unsafe { winuser::SendMessageW(self.rate.1, commctrl::TBM_GETPOS, 0, 0) } - 10;
                if self.settings.rate != new_rate as i32 {
                    changed = true;
                }
                // hotkeys change
                for (&(_, ht), hkt) in self.hotkeys.iter().zip_eq(self.settings.hotkeys.iter()) {
                    let set_to =
                        unsafe { winuser::SendMessageW(ht, commctrl::HKM_GETHOTKEY, 0, 0) };
                    let new = (
                        u32::from(convert_mod(minwindef::HIBYTE(set_to as u16))),
                        u32::from(minwindef::LOBYTE(set_to as u16)),
                    );
                    if *hkt != new {
                        changed = true;
                    }
                }
                if self.cleaners
                    .iter()
                    .any(|x| x.1 as isize == l_param || x.2 as isize == l_param)
                {
                    // cleaners change
                    for (cl, rexpar) in self.cleaners.iter_mut().zip(self.settings.cleaners.iter())
                    {
                        let (re, pal) = rexpar.to_parts();
                        let new_a = get_window_text(cl.1).as_string();
                        let new_b = get_window_text(cl.2).as_string();
                        if !new_a.is_empty() || !new_b.is_empty() {
                            if (new_a != re.as_str()) || (new_b != pal) {
                                cl.0 = Some(RegexCleanerPair::new(new_a, new_b).is_ok());
                            } else {
                                cl.0 = None;
                            }
                        }
                    }
                }
                changed = changed || self.cleaners.iter().any(|x| x.0.is_some());
                invalid = invalid || self.cleaners.iter().any(|x| x.0 == Some(false));
                enable_window(self.save, changed && !invalid);
                if saving && changed && !invalid {
                    use press_hotkey;
                    use Action;
                    self.settings.rate = new_rate as i32;
                    for (&(_, ht), hkt) in
                        self.hotkeys.iter().zip_eq(self.settings.hotkeys.iter_mut())
                    {
                        let set_to =
                            unsafe { winuser::SendMessageW(ht, commctrl::HKM_GETHOTKEY, 0, 0) };
                        *hkt = (
                            u32::from(convert_mod(minwindef::HIBYTE(set_to as u16))),
                            u32::from(minwindef::LOBYTE(set_to as u16)),
                        );
                    }
                    self.settings.cleaners = self.cleaners
                        .iter()
                        .map(|cl| {
                            let new_a = get_window_text(cl.1).as_string();
                            let new_b = get_window_text(cl.2).as_string();
                            RegexCleanerPair::new(new_a, new_b).unwrap()
                        })
                        .collect();
                    self.settings.to_file();
                    enable_window(self.save, false);
                    press_hotkey(Action::ReloadSettings);
                }
            }
            // TODO: add CLeaner button
            // TODO: remove CLeaner button
            // TODO: reorder CLeaner button
            // TODO: Reset botton
            _ => {}
        }
        None
    }
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            rate: 6,
            hotkeys: [
                (2, VK_OEM_2 as u32),      // ctrl-? key
                (7, VK_ESCAPE as u32),     // ctrl-alt-shift-esk
                (7, 0x52 as u32),          // ctrl-alt-shift-r
                (7, 0x53 as u32),          // ctrl-alt-shift-s
                (3, VK_OEM_2 as u32),      // ctrl-alt-?
                (2, VK_OEM_PERIOD as u32), // ctrl-.
                (3, VK_OEM_MINUS as u32),  // ctrl-alt--
                (3, VK_OEM_PLUS as u32),   // ctrl-alt-=
            ],
            cleaners: RegexCleanerPair::prep_list(&[
                (r"\s+", " "),
                (
                    concat!(
                        r"(https?://)?(?P<a>[-a-zA-Z0-9@:%._\+~#=]{2,256}",
                        r"\.[a-z]{2,6})\b[-a-zA-Z0-9@:%_\+.~#?&//=]{10,}"
                    ),
                    "link to $a",
                ),
                (
                    r"(?P<s>[0-9a-f]{6})([0-9]+[a-f]|[a-f]+[0-9])[0-9a-f]*",
                    "hash $s",
                ),
            ]).unwrap(),
            time_estimater: Default::default(),
        }
    }
    pub fn get_dir(&self) -> ::std::path::PathBuf {
        prefs_base_dir()
            .map(|mut p| {
                p.push("us");
                p.push("rust_reader");
                p.push("setings.prefs.json");
                p
            })
            .unwrap_or_default()
    }
    pub fn from_file() -> Settings {
        Settings::load(&APP_INFO, "setings").unwrap_or_else(|_| {
            println!("failed to lode settings.");
            Settings::new()
        })
    }
    pub fn reload_from_file(&mut self) -> bool {
        if let Ok(new) = Settings::load(&APP_INFO, "setings") {
            println!("reload settings.");
            *self = new;
            true
        } else {
            println!("failed to reload settings.");
            false
        }
    }
    pub fn to_file(&self) {
        if self.save(&APP_INFO, "setings").is_err() {
            println!("failed to save settings.");
        }
    }
}
