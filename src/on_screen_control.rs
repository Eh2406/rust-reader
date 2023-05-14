use crate::press_hotkey;
use crate::window::*;
use crate::Action;
use windows::core::PCWSTR;
use windows::w;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi,
    System::LibraryLoader,
    UI::WindowsAndMessaging as wm,
};

pub struct OnScreenControlWindow {
    window: HWND,
    read: HWND,
    pause: HWND,
}

impl OnScreenControlWindow {
    pub fn new() -> Box<OnScreenControlWindow> {
        let mut out = Box::new(OnScreenControlWindow {
            window: HWND(0),
            read: HWND(0),
            pause: HWND(0),
        });

        let window_class_name = w!("on_screen_control_window_class_name");
        unsafe {
            wm::RegisterClassW(&wm::WNDCLASSW {
                style: wm::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(window_proc_generic::<OnScreenControlWindow>),
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
                lpszClassName: window_class_name,
            });
            out.window = wm::CreateWindowExW(
                // WS_EX_NOACTIVATE makes window interactive but unfocusable,
                // like an on-screen keyboard
                wm::WS_EX_NOACTIVATE,
                window_class_name,
                w!(""),
                wm::WS_OVERLAPPED | wm::WS_SYSMENU,
                0,
                0,
                0,
                0,
                wm::GetDesktopWindow(),
                wm::HMENU(0),
                HINSTANCE(0),
                Some(&mut *out as *mut _ as *mut _),
            );
            // HWND_TOPMOST set window to always be on top
            wm::SetWindowPos(
                out.window,
                wm::HWND_TOPMOST,
                0,
                0,
                0,
                0,
                wm::SWP_NOMOVE | wm::SWP_NOSIZE,
            );
            out.read = create_button_window(out.window, w!("read"));
            out.pause = create_button_window(out.window, w!("pause/resume"));
        }
        set_window_text(out.window, &"reader controls".into());
        move_window(
            out.window,
            &RECT {
                left: 0,
                top: 80,
                right: 0,
                bottom: 0,
            },
        );
        out
    }

    pub fn show_window(&self) -> bool {
        show_window(self.window, wm::SW_SHOW)
    }
}

impl Windowed for OnScreenControlWindow {
    fn window_proc(&mut self, msg: u32, w_param: WPARAM, l_param: LPARAM) -> Option<LRESULT> {
        match msg {
            wm::WM_CLOSE => {
                show_window(self.window, wm::SW_HIDE);
                return Some(LRESULT(0));
            }
            wm::WM_SIZE => {
                let rect = get_client_rect(self.window).inset(3);
                if (w_param.0 <= 2) && rect.right > 0 && rect.bottom > 0 {
                    let rect = rect.split_rows(rect.bottom - 68);
                    let (l, r) = rect.1.split_columns(rect.1.right / 2);
                    move_window(self.read, &l);
                    move_window(self.pause, &r);
                    return Some(LRESULT(0));
                }
            }
            wm::WM_GETMINMAXINFO => {
                let data = unsafe { &mut *(l_param.0 as *mut wm::MINMAXINFO) };
                data.ptMinTrackSize.x = 240;
                data.ptMinTrackSize.y = 110;
                return Some(LRESULT(0));
            }
            wm::WM_COMMAND | wm::WM_HSCROLL => {
                let hiword = ((w_param.0 >> 16) & 0xffff) as u32;

                if hiword == wm::BN_CLICKED {
                    if l_param.0 == self.read.0 {
                        press_hotkey(Action::Read);
                    }
                    if l_param.0 == self.pause.0 {
                        press_hotkey(Action::PlayPause);
                    }
                }
            }
            _ => {}
        }
        None
    }
}
