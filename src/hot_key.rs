use user32;

use std::ptr::null_mut;

fn convert_modifiers(modifiers: u32) -> String {
    ["Alt", "Ctr", "Sht", "Win"]
        .iter()
        .enumerate()
        .filter(|&(i, _)| (modifiers & (1 << i)) > 0)
        .map(|(_, &val)| val)
        .collect()
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
    assert_eq!(&convert_modifiers(3), "AltCtr");
}

#[test]
fn test_modifiers_altctrsht() {
    assert_eq!(&convert_modifiers(7), "AltCtrSht");
}

pub struct HotKey {
    id: i32,
}

impl HotKey {
    pub fn new(modifiers: u32, vk: u32, id: i32) -> Option<HotKey> {
        use std::char;
        println!("new for HotKey: {}{} {}",
                 convert_modifiers(modifiers),
                 char::from_u32(unsafe { user32::MapVirtualKeyW(vk, 2) }).unwrap(),
                 id);
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms646309.aspx
        let hr = unsafe { user32::RegisterHotKey(null_mut(), id, modifiers, vk) };
        if hr == 0 {
            None
        } else {
            Some(HotKey { id: id })
        }
    }
}

impl Drop for HotKey {
    fn drop(&mut self) {
        unsafe { user32::UnregisterHotKey(null_mut(), self.id) };
        println!("drop for HotKey");
    }
}
