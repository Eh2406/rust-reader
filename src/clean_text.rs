use unicode_segmentation::*;
use std::ops::Range;

pub trait IsWhitespace {
    fn is_whitespace(&self) -> bool;
}

impl IsWhitespace for str {
    fn is_whitespace(&self) -> bool {
        self.chars().all(|x| x.is_whitespace())
    }
}

pub trait IsNumeric {
    fn is_numeric(&self) -> bool;
}

impl IsNumeric for str {
    fn is_numeric(&self) -> bool {
        self.chars().all(|x| x.is_numeric())
    }
}

fn runing_count<'a>(st: &mut (&'a str, usize), ch: &'a str) -> Option<&'a str> {
    let c_is_whitespace = ch.is_whitespace();
    if st.0 != ch && (!st.0.is_whitespace() || !c_is_whitespace) {
        st.1 = 0
    }
    st.1 += 1;
    st.0 = ch;
    if st.1 == 1 || st.1 < 4 && !c_is_whitespace || ch.is_numeric() {
        if c_is_whitespace {
            Some(" ")
        } else {
            Some(ch)
        }
    } else {
        Some("")
    }
}

pub fn clean_text<T: AsRef<str>>(raw: T) -> String {
    raw.as_ref()
       .graphemes(true)
       .scan(("", 0), runing_count)
       .collect()
}

pub fn clean_text_u8idx<T: AsRef<str>>(raw: T) -> Vec<(usize, usize)> {
    clean_text_idx(raw, |s| s.len())
}

pub fn clean_text_u16idx<T: AsRef<str>>(raw: T) -> Vec<(usize, usize)> {
    use wide_string::LenUtf;
    clean_text_idx(raw, |s| s.len_utf16())
}

fn clean_text_idx<T: AsRef<str>, F: Fn(&str) -> usize>(raw: T, len: F) -> Vec<(usize, usize)> {
    let st = raw.as_ref();
    let mut out = Vec::with_capacity(st.len());
    let mut scan = ("", 0);
    let mut in_idx = 0;
    let mut out_idx = 0;
    out.push((in_idx, out_idx));
    for gra in st.graphemes(true) {
        in_idx += len(gra);
        out_idx += len(runing_count(&mut scan, gra).unwrap());
        out.push((in_idx, out_idx));
    }
    out.shrink_to_fit();
    out
}

pub fn invert_idx(idx_vec: &Vec<(usize, usize)>, idx: Range<usize>) -> Range<usize> {
    // O(log(n))
    let mut start_idx = match idx_vec.binary_search_by(|&(_, x)| x.cmp(&idx.start)) {
        Ok(x) => x,
        Err(x) => x,
    };
    while start_idx > 0 && idx_vec[start_idx - 1].1 == idx.start {
        start_idx -= 1;
    }
    let start = idx_vec[start_idx].0;
    let mut end_idx = match idx_vec[start_idx..].binary_search_by(|&(_, x)| x.cmp(&idx.end)) {
        Ok(x) => x,
        Err(x) => x,
    } + start_idx;
    while end_idx < idx_vec.len() - 1 && idx_vec[end_idx + 1].1 == idx.end {
        end_idx += 1;
    }
    let end = idx_vec[end_idx].0;
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_word() {
        assert_eq!(clean_text("Hello"), "Hello");
    }

    #[test]
    fn one_word_u8idx() {
        let text = "Hello";
        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_idx(&vec_u8idx, 0..4), 0..4);
        assert_eq!(invert_idx(&vec_u8idx, 4..5), 4..5);
        assert_eq!(invert_idx(&vec_u8idx, 3..4), 3..4);
    }

    #[test]
    fn in_string() {
        assert_eq!(clean_text("Hello".to_string()), "Hello");
    }

    #[test]
    fn two_word() {
        assert_eq!(clean_text("Hello world!"), "Hello world!");
    }

    #[test]
    fn two_word_with_new_line() {
        assert_eq!(clean_text("Hello \t\n \t\r \t\r\n world!"), "Hello world!");
    }

    #[test]
    fn two_word_with_tabs() {
        assert_eq!(clean_text("Hello\t\n\t\r\t\r\nworld!"), "Hello world!");
    }

    #[test]
    fn two_word_with_tabs_u8idx() {
        let text = "Hello\t\n\t\r\t\r\nworld!";
        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_idx(&vec_u8idx, 5..6), 5..12);
        assert_eq!(invert_idx(&vec_u8idx, 6..7), 6..13);
        assert_eq!(invert_idx(&vec_u8idx, 3..7), 3..13);
        assert_eq!(invert_idx(&vec_u8idx, 3..8), 3..14);
    }

    #[test]
    fn two_word_with_underscore() {
        assert_eq!(clean_text("Hello _________ world!"), "Hello ___ world!");
    }

    #[test]
    fn two_word_with_underscore_u8idx() {
        let text = "Hello _________ world!";
        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_idx(&vec_u8idx, 7..9), 7..15);
        assert_eq!(invert_idx(&vec_u8idx, 9..10), 9..16);
        assert_eq!(invert_idx(&vec_u8idx, 8..12), 8..18);
        assert_eq!(invert_idx(&vec_u8idx, 11..15), 17..21);
    }

    #[test]
    fn two_word_with_dash() {
        assert_eq!(clean_text("Hello ----------- world!"), "Hello --- world!");
    }

    #[test]
    fn two_word_with_dash_u8idx() {
        assert_eq!(clean_text_u8idx("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} \
                                     ----------- \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} \
                                     world!"),
                   vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (10, 10),
                        (14, 14), (18, 18), (22, 18), (26, 18), (27, 19), (28, 20), (29, 21),
                        (30, 22), (31, 22), (32, 22), (33, 22), (34, 22), (35, 22), (36, 22),
                        (37, 22), (38, 22), (39, 23), (43, 27), (47, 31), (51, 35), (55, 35),
                        (59, 35), (60, 36), (61, 37), (62, 38), (63, 39), (64, 40), (65, 41),
                        (66, 42)]);
    }

    #[test]
    fn two_word_with_dash_u16idx() {
        assert_eq!(clean_text_u16idx("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} \
                                      ----------- \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} \
                                      world!"),
                   vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (8, 8), (10, 10),
                        (12, 12), (14, 12), (16, 12), (17, 13), (18, 14), (19, 15), (20, 16),
                        (21, 16), (22, 16), (23, 16), (24, 16), (25, 16), (26, 16), (27, 16),
                        (28, 16), (29, 17), (31, 19), (33, 21), (35, 23), (37, 23), (39, 23),
                        (40, 24), (41, 25), (42, 26), (43, 27), (44, 28), (45, 29), (46, 30)]);
    }

    #[test]
    fn two_word_with_equals() {
        assert_eq!(clean_text("Hello =========== world!"), "Hello === world!");
    }

    #[test]
    fn two_word_with_numbers() {
        assert_eq!(clean_text("Hello 100000 world!"), "Hello 100000 world!");
    }

    #[test]
    fn two_word_with_longchar() {
        assert_eq!(clean_text("Hello \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} world!"),
                   "Hello \u{1d565}\u{1d565}\u{1d565} world!");
    }

    #[test]
    fn two_word_with_longchar_u8idx() {
        let text = "Hello \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} world!";
        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_idx(&vec_u8idx, 5..6), 5..6);
        assert_eq!(invert_idx(&vec_u8idx, 6..18), 6..26);
        assert_eq!(invert_idx(&vec_u8idx, 18..20), 18..28);
        assert_eq!(invert_idx(&vec_u8idx, 14..24), 14..32);
    }

    #[test]
    fn two_word_with_multichar() {
        assert_eq!(clean_text("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\
                               \u{5d4}\u{5a2} world!"),
                   "Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} world!");
    }

    #[test]
    fn two_word_with_multichar_u8idx() {
        let text = "Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} \
                    world!";
        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_idx(&vec_u8idx, 6..20), 6..28);
        assert_eq!(invert_idx(&vec_u8idx, 18..19), 18..27);
        assert_eq!(invert_idx(&vec_u8idx, 14..18), 14..26);
        assert_eq!(invert_idx(&vec_u8idx, 14..22), 14..30);
    }

    fn test_clean_text_u8idx<T: AsRef<str>>(text: T) -> bool {
        let text = text.as_ref();
        for (in_idx, out_idx) in clean_text_u8idx(text) {
            if clean_text(&text[..in_idx]).len() != out_idx {
                println!("{:?}", clean_text_u8idx(text));
                println!("({:?}, {:?}) {:?}",
                         in_idx,
                         out_idx,
                         clean_text(&text[..in_idx]).len());
                return false;
            }
        }
        true
    }

    #[test]
    fn tests_clean_text_u8idx() {
        assert!(test_clean_text_u8idx("Hello"));
        assert!(test_clean_text_u8idx("Hello\t\n\t\r\t\r\nworld!"));
        assert!(test_clean_text_u8idx("Hello _________ world!"));
        assert!(test_clean_text_u8idx("Hello 100000 world!"));
        assert!(test_clean_text_u8idx("Hello \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} \
                                       world!"));
        assert!(test_clean_text_u8idx("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} \
                                       world!"));
    }

    #[test]
    fn quickcheck_clean_text_u8idx() {
        use quickcheck::quickcheck;
        quickcheck(test_clean_text_u8idx as fn(String) -> bool);
    }
}
