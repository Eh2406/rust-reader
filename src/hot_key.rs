use user32;

use std::ptr::null_mut;

pub struct HotKey {
    id: i32,
}

impl HotKey {
    pub fn new(modifiers: u32, vk: u32, id: i32) -> Option<HotKey> {
        println!("new for HotKey");
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
