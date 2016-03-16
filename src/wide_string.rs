use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ops::Range;

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16> {
        let mut out = self.to_wide();
        out.push(0);
        out
    }
}

impl<T: AsRef<OsStr>> ToWide for T {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
}

#[test]
fn one_larg_char() {
    let s = "\u{1d565}";
    assert_eq!(s.as_bytes(), [0xF0u8, 0x9D, 0x95, 0xA5]);
    assert_eq!(s.chars().collect::<Vec<char>>(), vec!['\u{1d565}']);
    assert_eq!(s.to_wide(), vec![0xD835, 0xDD65]);
    assert_eq!(s.to_wide_null(), vec![0xD835, 0xDD65, 0x0000]);

    assert_eq!(s.chars().map(|x| x.len_utf8()).collect::<Vec<usize>>(), vec![4]);
    assert_eq!(s.chars().map(|x| x.len_utf16()).collect::<Vec<usize>>(), vec![2]);
}

#[test]
fn two_small_char() {
    let s = "\u{5d4}\u{5a2}";
    assert_eq!(s.as_bytes(), [0xD7, 0x94, 0xD6, 0xA2]);
    assert_eq!(s.chars().collect::<Vec<char>>(), vec!['\u{5d4}', '\u{5a2}']);
    assert_eq!(s.to_wide(), vec![0x05D4, 0x05A2]);
    assert_eq!(s.to_wide_null(), vec![0x05D4, 0x05A2, 0x0000]);

    assert_eq!(s.chars().map(|x| x.len_utf8()).collect::<Vec<usize>>(), vec![2, 2]);
    assert_eq!(s.chars().map(|x| x.len_utf16()).collect::<Vec<usize>>(), vec![1, 1]);
}

#[allow(dead_code)]
pub fn string_from_str_u8idx(s: &str, idx: Range<usize>) -> String {
    // trivial
    s[idx].to_owned()
}

#[allow(dead_code)]
pub fn string_from_utf16_u16idx(s: &Vec<u16>, idx: Range<usize>) -> String {
    String::from_utf16_lossy(&s[idx])
}

#[allow(dead_code)]
pub fn string_from_str_u16idx(s: &str, idx: Range<usize>) -> String {
    string_from_utf16_u16idx(&s.to_wide(), idx)
}

#[allow(dead_code)]
pub fn string_from_utf16_u8idx(s: &Vec<u16>, idx: Range<usize>) -> String {
    String::from_utf16_lossy(&s)[idx].to_owned()
}
