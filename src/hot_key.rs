use user32;

use std::ptr::null_mut;

fn convert_modifiers(modifiers:u32)->String{
    let mut out = String::new();
    let mut modifiers = modifiers;
    if modifiers >= 4 {
        modifiers -= 4;
        out.push_str("Sht");
        }
    if modifiers >= 2 {
        modifiers -= 2;
        out.push_str("Ctr");
        }
    if modifiers >= 1 {
        modifiers -= 1;
        out.push_str("Alt");
        }
out
}

pub struct HotKey {
    id: i32,
}

impl HotKey {
    pub fn new(modifiers: u32, vk: u32, id: i32) -> Option<HotKey> {
        use std::char;
        println!("new for HotKey: {}{} {}",convert_modifiers(modifiers), char::from_u32(unsafe {user32::MapVirtualKeyW(vk,2)}).unwrap(), id);
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
