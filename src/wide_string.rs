use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct WideString(Vec<u16>);

impl WideString {
    pub fn new() -> WideString {
        WideString(vec![])
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a raw pointer to the slice's buffer.
    ///
    /// The caller must ensure that the slice outlives the pointer this
    /// function returns, or else it will end up pointing to garbage.
    ///
    /// Modifying the slice may cause its buffer to be reallocated, which
    /// would also make any pointers to it invalid.
    pub fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }
    pub fn get_slice(&self, range: Range<usize>) -> String {
        String::from_utf16_lossy(&self.0[range])
    }
}

impl<T: ::std::borrow::Borrow<str>> From<T> for WideString {
    fn from(instring: T) -> Self {
        let osstr: &OsStr = instring.borrow().as_ref();
        let mut out: Vec<u16> = osstr.encode_wide().collect();
        out.push(0);
        WideString(out)
    }
}

impl<T: ::std::borrow::Borrow<str>> ::std::iter::FromIterator<T> for WideString {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut out: Vec<u16> = Vec::new();
        for i in iter {
            let instr: &OsStr = i.borrow().as_ref();
            out.extend(instr.encode_wide());
        }
        out.push(0);
        WideString(out)
    }
}

impl Drop for WideString {
    fn drop(&mut self) {
        // this zeroes on drop so if we use freed memory we can see it
        for i in &mut self.0 {
            *i = 0;
        }
        assert_eq!(self.0.iter().sum::<u16>(), 0u16);
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
        self.chars().map(char::len_utf16).sum()
    }
}

pub trait IndicesUtf {
    fn indices_utf8(&self) -> Vec<usize>;
    fn indices_utf16(&self) -> Vec<usize>;
}

impl IndicesUtf for str {
    fn indices_utf8(&self) -> Vec<usize> {
        let mut out = vec![0];
        out.extend(self.chars().scan(0, |s, c| {
            *s += c.len_utf8();
            Some(*s)
        }));
        out
    }
    fn indices_utf16(&self) -> Vec<usize> {
        let mut out = vec![0];
        out.extend(self.chars().scan(0, |s, c| {
            *s += c.len_utf16();
            Some(*s)
        }));
        out
    }
}

#[allow(dead_code)]
pub fn convert_range<T>(v: &[T], r: &Range<T>) -> Range<usize>
where
    T: Ord,
{
    let mut s = v.binary_search(&r.start).unwrap_or_else(|x| x);
    while s > 0 && v[s - 1] == r.start {
        s -= 1;
    }
    let mut e = v[s..].binary_search(&r.end).unwrap_or_else(|x| x) + s;
    while e < v.len() - 1 && v[e + 1] == r.end {
        e += 1;
    }
    s..e
}

#[allow(dead_code)]
pub fn lookup_range<T>(v: &[T], r: &Range<usize>) -> Range<T>
where
    T: Clone,
{
    v[r.start].clone()..v[r.end].clone()
}

#[allow(dead_code)]
pub fn invert_idx<I, O>(i: &[I], o: &[O], r: &Range<O>) -> Range<I>
where
    O: Ord,
    I: Clone,
{
    assert_eq!(i.len(), o.len());
    lookup_range(i, &convert_range(o, r))
}

#[allow(dead_code)]
pub fn str_from_str_u16idx(s: &str, idx: Range<usize>) -> &str {
    &s[u8idx_from_u16idx(s, idx)]
}

#[allow(dead_code)]
pub fn u8idx_from_u16idx(s: &str, idx: Range<usize>) -> Range<usize> {
    let mut u16idx = 0;
    let mut out = 0..0;
    for c in s.chars() {
        if u16idx <= idx.start {
            out.start = out.end;
            println!("{:?}", out.start);
        }
        out.end += c.len_utf8();
        u16idx += c.len_utf16();
        if idx.end <= u16idx {
            return out;
        }
    }
    out
}

#[allow(dead_code)]
pub fn u16idx_from_u8idx(s: &str, idx: Range<usize>) -> Range<usize> {
    let start = s[..idx.start].len_utf16();
    start..start + s[idx].len_utf16()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_larg_char() {
        let s = "\u{1d565}";
        assert_eq!(s.as_bytes(), [0xF0u8, 0x9D, 0x95, 0xA5]);
        assert_eq!(s.chars().collect::<Vec<char>>(), vec!['\u{1d565}']);
        assert_eq!(
            WideString::from(s),
            WideString(vec![0xD835, 0xDD65, 0x0000])
        );

        assert_eq!(s.indices_utf8(), vec![0, 4]);
        assert_eq!(s.indices_utf16(), vec![0, 2]);
    }

    #[test]
    fn two_small_char() {
        let s = "\u{5d4}\u{5a2}";
        assert_eq!(s.as_bytes(), [0xD7, 0x94, 0xD6, 0xA2]);
        assert_eq!(s.chars().collect::<Vec<char>>(), vec!['\u{5d4}', '\u{5a2}']);
        assert_eq!(
            WideString::from(s),
            WideString(vec![0x05D4, 0x05A2, 0x0000])
        );

        assert_eq!(s.indices_utf8(), vec![0, 2, 4]);
        assert_eq!(s.indices_utf16(), vec![0, 1, 2]);
    }

    #[test]
    fn u16idx() {
        let s = "\u{1d565} \u{5d4}\u{5a2} \u{1d565} \u{5d4}\u{5a2}";

        assert_eq!(0..5, u16idx_from_u8idx(s, 0..9));
        assert_eq!(2..5, u16idx_from_u8idx(s, 4..9));
        assert_eq!(2..8, u16idx_from_u8idx(s, 4..14));
        assert_eq!(2..11, u16idx_from_u8idx(s, 4..19));

        assert_eq!(0..9, u8idx_from_u16idx(s, u16idx_from_u8idx(s, 0..9)));
        assert_eq!(4..9, u8idx_from_u16idx(s, u16idx_from_u8idx(s, 4..9)));
        assert_eq!(4..14, u8idx_from_u16idx(s, u16idx_from_u8idx(s, 4..14)));

        assert_eq!(0..5, u16idx_from_u8idx(s, u8idx_from_u16idx(s, 0..5)));
        assert_eq!(2..5, u16idx_from_u8idx(s, u8idx_from_u16idx(s, 2..5)));
        assert_eq!(2..8, u16idx_from_u8idx(s, u8idx_from_u16idx(s, 2..8)));

        assert_eq!(&s[0..9], str_from_str_u16idx(s, 0..5));
        assert_eq!(&s[4..9], str_from_str_u16idx(s, 2..5));
        assert_eq!(&s[4..14], str_from_str_u16idx(s, 2..8));
        assert_eq!(&s[4..19], str_from_str_u16idx(s, 2..11));

        let id8 = s.indices_utf8();
        let id16 = s.indices_utf16();
        assert_eq!(0..5, invert_idx(&id16, &id8, &(0..9)));
        assert_eq!(2..5, invert_idx(&id16, &id8, &(4..9)));
        assert_eq!(2..8, invert_idx(&id16, &id8, &(4..14)));
        assert_eq!(2..11, invert_idx(&id16, &id8, &(4..19)));

        assert_eq!(0..9, invert_idx(&id8, &id16, &(0..5)));
        assert_eq!(4..9, invert_idx(&id8, &id16, &(2..5)));
        assert_eq!(4..14, invert_idx(&id8, &id16, &(2..8)));
        assert_eq!(4..19, invert_idx(&id8, &id16, &(2..11)));
    }
}
