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

pub trait LenUtf {
    fn len_utf8(&self) -> usize;
    fn len_utf16(&self) -> usize;
}

impl LenUtf for str {
    fn len_utf8(&self) -> usize {
        self.len()
    }
    fn len_utf16(&self) -> usize {
        self.chars().map(|x| x.len_utf16()).fold(0, |s, e| s + e)
    }
}

pub trait IndicesUtf {
    fn indices_utf8(&self) -> Vec<usize>;
    fn indices_utf16(&self) -> Vec<usize>;
}

impl IndicesUtf for str {
    fn indices_utf8(&self) -> Vec<usize> {
        self.chars()
            .scan(0, |s, c| {
                *s += c.len_utf8();
                Some(*s)
            })
            .collect()
    }
    fn indices_utf16(&self) -> Vec<usize> {
        self.chars()
            .scan(0, |s, c| {
                *s += c.len_utf16();
                Some(*s)
            })
            .collect()
    }
}

fn convert_range<T>(v: &[T], r: &Range<T>) -> Range<usize>
    where T: Ord
{
    let s = match v.binary_search(&r.start) {
        Ok(x) => x,
        Err(x) => x,
    };
    let e = match v[s..].binary_search(&r.end) {
        Ok(x) => x,
        Err(x) => x,
    };

    s..s + e
}

fn lookup_range<T>(v: &[T], r: &Range<usize>) -> Range<T>
    where T: Copy
{
    v[r.start]..v[r.end]
}

#[test]
fn one_larg_char() {
    let s = "\u{1d565}";
    assert_eq!(s.as_bytes(), [0xF0u8, 0x9D, 0x95, 0xA5]);
    assert_eq!(s.chars().collect::<Vec<char>>(), vec!['\u{1d565}']);
    assert_eq!(s.to_wide(), vec![0xD835, 0xDD65]);
    assert_eq!(s.to_wide_null(), vec![0xD835, 0xDD65, 0x0000]);

    assert_eq!(s.indices_utf8(), vec![4]);
    assert_eq!(s.indices_utf16(), vec![2]);
}

#[test]
fn two_small_char() {
    let s = "\u{5d4}\u{5a2}";
    assert_eq!(s.as_bytes(), [0xD7, 0x94, 0xD6, 0xA2]);
    assert_eq!(s.chars().collect::<Vec<char>>(), vec!['\u{5d4}', '\u{5a2}']);
    assert_eq!(s.to_wide(), vec![0x05D4, 0x05A2]);
    assert_eq!(s.to_wide_null(), vec![0x05D4, 0x05A2, 0x0000]);

    assert_eq!(s.indices_utf8(), vec![2, 4]);
    assert_eq!(s.indices_utf16(), vec![1, 2]);
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
    String::from_utf16_lossy(&s.to_wide()[idx])
}

#[allow(dead_code)]
pub fn string_from_utf16_u8idx(s: &Vec<u16>, idx: Range<usize>) -> String {
    String::from_utf16_lossy(&s)[idx].to_owned()
}
