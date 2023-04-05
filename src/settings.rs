use crate::clean_text::RegexCleanerPair;
use crate::hot_key::*;
use crate::wide_string::WideString;
use crate::window::*;
use average::Variance;
use itertools::Itertools;
use preferences::{prefs_base_dir, AppInfo, Preferences};
use windows::core::PCWSTR;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi,
    System::LibraryLoader,
    UI::Controls,
    UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_OEM_2, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS},
    UI::WindowsAndMessaging as wm,
};

const APP_INFO: AppInfo = AppInfo {
    name: "rust_reader",
    author: "us",
};

// TBM_SETPOS is defined in winrows crate, but TBM_GETPOS is missing?
pub const TBM_GETPOS: u32 = Controls::TBM_SETPOS - 5;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub rate: i32,
    pub hotkeys: [(u32, u32); 8],
    pub cleaners: Vec<RegexCleanerPair>,
    #[serde(default)]
    pub time_estimater: [Variance; 21],
}

pub struct SettingsWindow {
    settings: Settings,
    window: HWND,
    rate: (HWND, HWND),
    hotkeys: [(HWND, HWND); 8],
    cleaners: Vec<(Option<bool>, HWND, HWND, HWND, HWND)>,
    add_cleaner: HWND,
    reset: HWND,
    save: HWND,
}

impl SettingsWindow {
    pub fn new(s: Settings) -> Box<SettingsWindow> {
        let mut out = Box::new(SettingsWindow {
            settings: s,
            window: HWND(0),
            rate: (HWND(0), HWND(0)),
            hotkeys: [(HWND(0), HWND(0)); 8],
            cleaners: Vec::new(),
            add_cleaner: HWND(0),
            reset: HWND(0),
            save: HWND(0),
        });

        let window_class_name: WideString = "setings_window_class_name".into();
        unsafe {
            wm::RegisterClassW(&wm::WNDCLASSW {
                style: wm::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(window_proc_generic::<SettingsWindow>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: HINSTANCE(0),
                hIcon: wm::LoadIconW(
                    LibraryLoader::GetModuleHandleW(PCWSTR::null()).unwrap(),
                    PCWSTR::from_raw(1 as *const u16),
                )
                .expect("failed to load icon"),
                hCursor: wm::LoadCursorW(HINSTANCE(0), wm::IDI_APPLICATION)
                    .expect("failed to load icon"),
                hbrBackground: Gdi::HBRUSH(16),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: PCWSTR::from_raw(window_class_name.as_ptr()),
            });
            out.window = wm::CreateWindowExW(
                wm::WINDOW_EX_STYLE(0),
                PCWSTR::from_raw(window_class_name.as_ptr()),
                PCWSTR(&mut 0u16),
                wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPSIBLINGS,
                0,
                0,
                0,
                0,
                wm::GetDesktopWindow(),
                wm::HMENU(0),
                HINSTANCE(0),
                Some(&mut *out as *mut _ as *mut _),
            );
            Controls::InitCommonControls();
            let wide_trackbar: WideString = "msctls_trackbar32".into();
            out.rate.1 = wm::CreateWindowExW(
                wm::WINDOW_EX_STYLE(0),
                PCWSTR::from_raw(wide_trackbar.as_ptr()),
                PCWSTR(&mut 0u16),
                wm::WS_CHILD
                    | wm::WS_VISIBLE
                    | wm::WINDOW_STYLE(Controls::TBS_AUTOTICKS | Controls::TBS_BOTTOM),
                0,
                0,
                0,
                0,
                out.window,
                wm::HMENU(0),
                HINSTANCE(0),
                None,
            );
            wm::SendMessageW(
                out.rate.1,
                Controls::TBM_SETRANGE,
                WPARAM(0),
                LPARAM((20 << 16) as isize),
            );
            wm::SendMessageW(out.rate.1, Controls::TBM_SETPAGESIZE, WPARAM(0), LPARAM(1));
            out.rate.0 = create_static_window(out.window, None);

            out.add_cleaner = create_button_window(out.window, Some(&"add cleaner".into()));
            out.save = create_button_window(out.window, Some(&"save".into()));
            out.reset = create_button_window(out.window, Some(&"reset".into()));
            let window = out.window;
            let wide_hotkey_class: WideString = "msctls_hotkey32".into();

            let mut icex: Controls::INITCOMMONCONTROLSEX = ::std::mem::zeroed();
            icex.dwSize = ::std::mem::size_of::<Controls::INITCOMMONCONTROLSEX>() as u32;
            icex.dwICC = Controls::ICC_HOTKEY_CLASS;
            Controls::InitCommonControlsEx(&icex);

            for (act, ht) in crate::actions::ACTION_LIST
                .iter()
                .zip(out.hotkeys.iter_mut())
            {
                let wide_hotkey_name: WideString = format!("{}", act).into();
                ht.0 = create_static_window(window, Some(&wide_hotkey_name));
                ht.1 = wm::CreateWindowExW(
                    wm::WINDOW_EX_STYLE(0),
                    PCWSTR::from_raw(wide_hotkey_class.as_ptr()),
                    PCWSTR(&mut 0u16),
                    wm::WS_CHILD | wm::WS_VISIBLE,
                    0,
                    0,
                    0,
                    0,
                    window,
                    wm::HMENU(0),
                    HINSTANCE(0),
                    None,
                );
                wm::SendMessageW(
                    ht.1,
                    Controls::HKM_SETRULES,
                    WPARAM((Controls::HKCOMB_NONE | Controls::HKCOMB_S) as usize),
                    LPARAM(Controls::HOTKEYF_CONTROL as isize),
                );
            }
        }
        set_window_text(out.window, &"reader settings".into());
        out.get_inner_all();
        move_window(
            out.window,
            &RECT {
                left: 0,
                top: 0,
                right: 400,
                bottom: 400,
            },
        );
        show_window(out.window, wm::SW_SHOWNORMAL);
        out.toggle_window_visible();
        out
    }

    fn add_cleaner(&mut self) {
        self.cleaners.push((
            None,
            create_edit_window(self.window, wm::WINDOW_STYLE(0)),
            create_edit_window(self.window, wm::WINDOW_STYLE(0)),
            create_button_window(self.window, Some(&"^".into())),
            create_button_window(self.window, Some(&"X".into())),
        ));
    }

    fn remove_cleaner(&mut self, index: usize) {
        let hwnd = self.cleaners.remove(index);
        destroy_window(hwnd.1);
        destroy_window(hwnd.2);
        destroy_window(hwnd.3);
        destroy_window(hwnd.4);
    }

    fn swap_cleaner(&mut self, index: usize) {
        if index >= 1 {
            self.cleaners.swap(index - 1, index);
        }
    }

    pub fn get_inner_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn get_mut_inner_settings(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn toggle_window_visible(&self) -> bool {
        toggle_window_visible(self.window)
    }

    pub fn show_window(&self) -> bool {
        show_window(self.window, wm::SW_SHOW)
    }

    pub fn get_inner_rate(&mut self) -> i32 {
        let rate = self.settings.rate;
        unsafe {
            wm::SendMessageW(
                self.rate.1,
                Controls::TBM_SETPOS,
                WPARAM(1),
                LPARAM((rate + 10) as isize),
            );
        }
        set_window_text(self.rate.0, &format!("reading at rate: {}", rate).into());
        rate
    }

    pub fn get_inner_hotkeys(&self) -> [(u32, u32); 8] {
        for (&(a, b), hwnd) in self.settings.hotkeys.iter().zip(self.hotkeys.iter()) {
            unsafe {
                wm::SendMessageW(
                    hwnd.1,
                    Controls::HKM_SETHOTKEY,
                    WPARAM((b as u16 | ((convert_mod(a as u8) as u16) << 8)).into()),
                    LPARAM(0),
                );
            }
        }
        self.settings.hotkeys
    }

    pub fn get_inner_cleaners(&mut self) -> &[RegexCleanerPair] {
        if self.cleaners.len() != self.settings.cleaners.len() {
            while self.cleaners.len() < self.settings.cleaners.len() {
                self.add_cleaner();
            }
            while self.cleaners.len() > self.settings.cleaners.len() {
                let i = self.cleaners.len() - 1;
                self.remove_cleaner(i);
            }
            unsafe {
                wm::SendMessageW(self.window, wm::WM_SIZE, WPARAM(0), LPARAM(0));
            }
        }
        for (cl, rexpar) in self
            .cleaners
            .iter_mut()
            .zip_eq(self.settings.cleaners.iter())
        {
            let (re, pal) = rexpar.to_parts();
            cl.0 = None;
            set_window_text(cl.1, &re.as_str().into());
            set_window_text(cl.2, &pal.into());
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
    fn window_proc(&mut self, msg: u32, w_param: WPARAM, l_param: LPARAM) -> Option<LRESULT> {
        use itertools::EitherOrBoth::{Both, Left, Right};
        match msg {
            wm::WM_CLOSE => {
                show_window(self.window, wm::SW_HIDE);
                return Some(LRESULT(0));
            }
            wm::WM_SIZE => {
                let rect = get_client_rect(self.window).inset(3);
                if (w_param.0 <= 2) && rect.right > 0 && rect.bottom > 0 {
                    let mut rect = rect.split_rows(rect.bottom - 50);
                    let mut bot = rect.1.split_rows(25);
                    bot.0 = bot.0.inset(3).shift_right(50);
                    bot.0.right -= 50;
                    move_window(self.add_cleaner, &bot.0);
                    let (l, r) = bot.1.split_columns(bot.1.right / 2);
                    move_window(self.reset, &l);
                    move_window(self.save, &r);
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
                    let mll = self
                        .cleaners
                        .iter()
                        .map(|&(_, a, _, _, _)| get_window_text_length(a))
                        .max()
                        .unwrap_or(0)
                        + 1;
                    let mlr = self
                        .cleaners
                        .iter()
                        .map(|&(_, _, b, _, _)| get_window_text_length(b))
                        .max()
                        .unwrap_or(0)
                        + 1;
                    rect.1 = rect.1.shift_down(5);
                    let split_at = (rect.1.right - 50) * mll / (mll + mlr);
                    for &ht in &self.cleaners {
                        rect = rect.1.split_rows(25);
                        let (l, r) = rect.0.split_columns(rect.1.right - 50);
                        let r = r.split_columns(25);
                        unsafe {
                            Gdi::InvalidateRect(ht.3, None, true);
                        }
                        move_window(ht.3, &r.0.inset(3));
                        unsafe {
                            Gdi::InvalidateRect(ht.4, None, true);
                        }
                        move_window(ht.4, &r.1.inset(3));
                        let (l, r) = l.split_columns(split_at);
                        move_window(ht.1, &l);
                        move_window(ht.2, &r);
                    }
                    return Some(LRESULT(0));
                }
            }
            wm::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param.0 as *mut wm::MINMAXINFO) };
                data.ptMinTrackSize.x = 340;
                data.ptMinTrackSize.y =
                    (55 + 25 * (3 + self.hotkeys.len()) + 25 * self.cleaners.len()) as i32;
                return Some(LRESULT(0));
            }
            wm::WM_COMMAND | wm::WM_HSCROLL => {
                let mut changed = false;
                let mut invalid = false;
                let mut dirty_cleaners = false;

                if ((w_param.0 >> 16) & 0xffff) as u32 == wm::BN_CLICKED {
                    if self.reset.0 == l_param.0 {
                        self.get_inner_all();
                    }
                    if self.add_cleaner.0 == l_param.0 {
                        self.add_cleaner();
                        dirty_cleaners = true;
                        unsafe {
                            wm::SendMessageW(self.window, wm::WM_SIZE, WPARAM(0), LPARAM(0));
                        }
                    }
                    if let Some(i) = self.cleaners.iter().position(|x| x.3 .0 == l_param.0) {
                        self.swap_cleaner(i);
                        dirty_cleaners = true;
                        unsafe {
                            wm::SendMessageW(self.window, wm::WM_SIZE, WPARAM(0), LPARAM(0));
                        }
                    }
                    if let Some(i) = self.cleaners.iter().position(|x| x.4 .0 == l_param.0) {
                        self.remove_cleaner(i);
                        dirty_cleaners = true;
                        unsafe {
                            wm::SendMessageW(self.window, wm::WM_SIZE, WPARAM(0), LPARAM(0));
                        }
                    }
                }

                let saving = self.save.0 == l_param.0
                    && ((w_param.0 >> 16) & 0xffff) as u32 == wm::BN_CLICKED;

                // rate change
                let new_rate =
                    unsafe { wm::SendMessageW(self.rate.1, TBM_GETPOS, WPARAM(0), LPARAM(0)) }.0
                        - 10;
                if self.settings.rate != new_rate as i32 {
                    changed = true;
                }
                // hotkeys change
                for (&(_, ht), hkt) in self.hotkeys.iter().zip_eq(self.settings.hotkeys.iter()) {
                    let set_to = unsafe {
                        wm::SendMessageW(ht, Controls::HKM_GETHOTKEY, WPARAM(0), LPARAM(0))
                    }
                    .0;
                    let new = (
                        u32::from(convert_mod(((set_to >> 8) & 0xff) as u8)),
                        u32::from((set_to as u16) & 0xff),
                    );
                    if *hkt != new {
                        changed = true;
                    }
                }
                if self
                    .cleaners
                    .iter()
                    .any(|x| x.1 .0 == l_param.0 || x.2 .0 == l_param.0)
                    || dirty_cleaners
                {
                    // cleaners change
                    for mat in self
                        .cleaners
                        .iter_mut()
                        .zip_longest(self.settings.cleaners.iter())
                    {
                        match mat {
                            Both(cl, rexpar) => {
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
                            Right(_) => (),
                            Left(cl) => {
                                let new_a = get_window_text(cl.1).as_string();
                                let new_b = get_window_text(cl.2).as_string();
                                if !new_a.is_empty() || !new_b.is_empty() {
                                    cl.0 = Some(RegexCleanerPair::new(new_a, new_b).is_ok());
                                }
                            }
                        }
                    }
                }
                changed = changed
                    || self.settings.cleaners.len() != self.cleaners.len()
                    || self.cleaners.iter().any(|x| x.0.is_some());
                invalid = invalid || self.cleaners.iter().any(|x| x.0 == Some(false));
                enable_window(self.reset, changed);
                enable_window(self.save, changed && !invalid);
                if saving && changed && !invalid {
                    use crate::press_hotkey;
                    use crate::Action;
                    self.settings.rate = new_rate as i32;
                    for (&(_, ht), hkt) in
                        self.hotkeys.iter().zip_eq(self.settings.hotkeys.iter_mut())
                    {
                        let set_to = unsafe {
                            wm::SendMessageW(ht, Controls::HKM_GETHOTKEY, WPARAM(0), LPARAM(0))
                        }
                        .0;
                        *hkt = (
                            u32::from(convert_mod(((set_to >> 8) & 0xff) as u8)),
                            u32::from((set_to as u16) & 0xff),
                        );
                    }
                    self.settings.cleaners = self
                        .cleaners
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
                (2, VK_OEM_2.0.into()),      // ctrl-? key
                (7, VK_ESCAPE.0.into()),     // ctrl-alt-shift-esk
                (7, 0x52 as u32),            // ctrl-alt-shift-r
                (7, 0x53 as u32),            // ctrl-alt-shift-s
                (3, VK_OEM_2.0.into()),      // ctrl-alt-?
                (2, VK_OEM_PERIOD.0.into()), // ctrl-.
                (3, VK_OEM_MINUS.0.into()),  // ctrl-alt--
                (3, VK_OEM_PLUS.0.into()),   // ctrl-alt-=
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
            ])
            .unwrap(),
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
