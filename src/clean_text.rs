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
    let st = raw.as_ref();
    let mut out = Vec::with_capacity(st.len());
    let mut scan = ("", 0);
    let mut in_idx = 0;
    let mut out_idx = 0;
    out.push((in_idx, out_idx));
    for gra in st.graphemes(true) {
        in_idx += gra.len();
        out_idx += runing_count(&mut scan, gra).unwrap().len();
        out.push((in_idx, out_idx));
    }
    out.shrink_to_fit();
    out
}

pub fn invert_u8idx(idx_vec: &Vec<(usize, usize)>, idx: Range<usize>) -> Range<usize> {
    // O(n)
    let start = idx_vec.iter().skip_while(|&&(_, x)| x < idx.start).map(|&(x, _)| x).next();
    let end = idx_vec.iter().rev().skip_while(|&&(_, x)| x > idx.end).map(|&(x, _)| x).next();
    start.unwrap()..end.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;
    use wide_string::*;

    fn slow_invert_u8idx<T: AsRef<str>>(raw: T, idx: Range<usize>) -> Range<usize> {
        // O(n^2)
        let s = raw.as_ref();
        let mut out = 0..s.len() + 1;
        for x in s.indices_utf8() {
            let l = clean_text(&s[..x]).len();
            if l == idx.start {
                out.start = x;
            }
            if l == idx.end {
                out.end = x;
            }
        }
        out
    }

    #[test]
    fn one_word() {
        assert_eq!(clean_text("Hello"), "Hello");
    }

    #[test]
    fn one_word_u8idx() {
        let text = "Hello";
        assert_eq!(slow_invert_u8idx(text, 0..5), 0..5);
        assert_eq!(slow_invert_u8idx(text, 0..4), 0..4);
        assert_eq!(slow_invert_u8idx(text, 4..5), 4..5);
        assert_eq!(slow_invert_u8idx(text, 3..4), 3..4);

        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_u8idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 0..4), 0..4);
        assert_eq!(invert_u8idx(&vec_u8idx, 4..5), 4..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..4), 3..4);
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
        assert_eq!(slow_invert_u8idx(text, 0..5), 0..5);
        assert_eq!(slow_invert_u8idx(text, 3..5), 3..5);
        assert_eq!(slow_invert_u8idx(text, 3..7), 3..13);
        assert_eq!(slow_invert_u8idx(text, 3..8), 3..14);

        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_u8idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..7), 3..13);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..8), 3..14);
    }

    #[test]
    fn two_word_with_underscore() {
        assert_eq!(clean_text("Hello _________ world!"), "Hello ___ world!");
    }

    #[test]
    fn two_word_with_underscore_u8idx() {
        let text = "Hello _________ world!";
        assert_eq!(slow_invert_u8idx(text, 0..5), 0..5);
        assert_eq!(slow_invert_u8idx(text, 3..5), 3..5);
        assert_eq!(slow_invert_u8idx(text, 8..12), 8..18);
        assert_eq!(slow_invert_u8idx(text, 11..15), 17..21);

        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_u8idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 8..12), 8..18);
        assert_eq!(invert_u8idx(&vec_u8idx, 11..15), 17..21);
    }

    #[test]
    fn two_word_with_dash() {
        assert_eq!(clean_text("Hello ----------- world!"), "Hello --- world!");
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
        assert_eq!(slow_invert_u8idx(text, 0..5), 0..5);
        assert_eq!(slow_invert_u8idx(text, 3..5), 3..5);
        assert_eq!(slow_invert_u8idx(text, 6..20), 6..28);
        assert_eq!(slow_invert_u8idx(text, 14..24), 14..32);

        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_u8idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 6..20), 6..28);
        assert_eq!(invert_u8idx(&vec_u8idx, 14..24), 14..32);
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
        assert_eq!(slow_invert_u8idx(text, 0..5), 0..5);
        assert_eq!(slow_invert_u8idx(text, 3..5), 3..5);
        assert_eq!(slow_invert_u8idx(text, 6..20), 6..28);
        assert_eq!(slow_invert_u8idx(text, 14..22), 14..30);

        let vec_u8idx = clean_text_u8idx(text);
        println!("{:?}", vec_u8idx);
        assert_eq!(invert_u8idx(&vec_u8idx, 0..5), 0..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 3..5), 3..5);
        assert_eq!(invert_u8idx(&vec_u8idx, 6..20), 6..28);
        assert_eq!(invert_u8idx(&vec_u8idx, 14..22), 14..30);
    }

    fn test_clean_text_u8idx(text: &str) {
        println!("{:?}", clean_text_u8idx(text));
        for (in_idx, out_idx) in clean_text_u8idx(text) {
            assert_eq!(clean_text(&text[..in_idx]).len(), out_idx);
        }
    }

    #[test]
    fn tests_clean_text_u8idx() {
        test_clean_text_u8idx("Hello");
        test_clean_text_u8idx("Hello\t\n\t\r\t\r\nworld!");
        test_clean_text_u8idx("Hello _________ world!");
        test_clean_text_u8idx("Hello \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} world!");
        test_clean_text_u8idx("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\
        \u{5d4}\u{5a2} world!");
    }
}
