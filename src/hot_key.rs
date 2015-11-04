use user32;

use std::ptr;

pub struct HotKey {
    id: i32,
}

impl HotKey {
    pub fn new(modifiers: u32, vk: u32, id: i32) -> HotKey {
        println!("new for HotKey");
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms646309.aspx
        unsafe {
            user32::RegisterHotKey(ptr::null_mut(), id, modifiers, vk);
        }
        HotKey { id: id }
    }
}

impl Drop for HotKey {
    fn drop(&mut self) {
        unsafe {
            user32::UnregisterHotKey(ptr::null_mut(), self.id);
        }
        println!("drop for HotKey");
    }
}
