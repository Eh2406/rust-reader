use user32;

use std::ptr::null_mut;

fn convert_modifiers(modifiers: u32) -> String {
    let mut out = String::new();
    if (modifiers & 4) > 0 {
        out.push_str("Sht");
    }
    if (modifiers & 2) > 0 {
        out.push_str("Ctr");
    }
    if (modifiers & 1) > 0 {
        out.push_str("Alt");
    }
    out
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
fn test_modifiers_ctralt() {
    assert_eq!(&convert_modifiers(3), "CtrAlt");
}

#[test]
fn test_modifiers_ctraltsht() {
    assert_eq!(&convert_modifiers(7), "ShtCtrAlt");
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
