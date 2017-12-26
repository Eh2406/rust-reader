use winapi::um::winuser;
use winapi::um::winuser::VK_ESCAPE;
use itertools::Itertools;

use std::ptr::null_mut;

fn convert_modifiers(modifiers: u32) -> String {
    ["Alt", "Ctr", "Sht", "Win"]
        .iter()
        .enumerate()
        .filter(|&(i, _)| (modifiers & (1 << i)) > 0)
        .map(|(_, &val)| val)
        .join("+")
}

#[test]
fn test_modifiers_alt() {
    assert_eq!(&convert_modifiers(1), "Alt");
}

#[test]
fn test_modifiers_ctr() {
    assert_eq!(&convert_modifiers(2), "Ctr");
}

#[test]
fn test_modifiers_sht() {
    assert_eq!(&convert_modifiers(4), "Sht");
}

#[test]
fn test_modifiers_altctr() {
    assert_eq!(&convert_modifiers(3), "Alt+Ctr");
}

#[test]
fn test_modifiers_altctrsht() {
    assert_eq!(&convert_modifiers(7), "Alt+Ctr+Sht");
}

pub fn convert_mod(modifiers: u8) -> u8 {
    // the Modifiers is different between RegisterHotKey and HKM_SETHOTKEY
    let mut to_modifiers = modifiers & !(5);
    if (modifiers & 1) > 0 {
        to_modifiers |= 4;
    }
    if (modifiers & 4) > 0 {
        to_modifiers |= 1;
    }
    to_modifiers
}

#[derive(Debug)]
pub struct HotKey {
    vk: u32,
    modifiers: u32,
    id: i32,
}

impl HotKey {
    pub fn new(modifiers: u32, vk: u32, id: i32) -> Option<HotKey> {
        let new_hot = HotKey {
            modifiers: modifiers,
            vk: vk,
            id: id,
        };
        println!("new for HotKey: {} {}", new_hot, id);
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms646309.aspx
        if modifiers > 0 && vk > 0 {
            let hr = unsafe { winuser::RegisterHotKey(null_mut(), id, modifiers, vk) };
            if hr == 0 {
                None
            } else {
                Some(new_hot)
            }
        } else {
            Some(new_hot)
        }
    }
}

impl ::std::fmt::Display for HotKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use std::char;
        if self.modifiers > 0 && self.vk > 0 {
            write!(f, "{}+", convert_modifiers(self.modifiers))?;
            if self.vk == VK_ESCAPE as u32 {
                write!(f, "Esc")
            } else {
                write!(
                    f,
                    "{}",
                    char::from_u32(unsafe { winuser::MapVirtualKeyW(self.vk, 2) }).unwrap()
                )
            }
        } else {
            write!(f, "None")
        }
    }
}

impl Drop for HotKey {
    fn drop(&mut self) {
        if self.modifiers > 0 && self.vk > 0 {
            unsafe { winuser::UnregisterHotKey(null_mut(), self.id) };
        }
        println!("drop for HotKey");
    }
}
