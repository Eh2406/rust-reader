use unicode_segmentation::*;
use wide_string::*;
use std::borrow::Cow;
use regex::*;

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

fn regex_replacen<'a, R: Replacer>(re: &Regex,
                                   mut rep: R,
                                   text: &'a str)
                                   -> Vec<(&'a str, Cow<'a, str>)> {
    let mut new = Vec::new();
    let mut last_match = 0;
    for cap in re.captures_iter(text) {
        // unwrap on 0 is OK because captures only reports matches
        let (s, e) = cap.pos(0).unwrap();
        new.push((&text[last_match..s], text[last_match..s].into()));
        new.push((&text[s..e], rep.reg_replace(&cap).to_string().into()));
        last_match = e;
    }
    new.push((&text[last_match..], text[last_match..].into()));
    new
}

fn runing_count<'a>(st: &mut (Cow<'a, str>, usize),
                    (orig, ch): (&'a str, Cow<'a, str>))
                    -> Option<(&'a str, Cow<'a, str>)> {
    if orig != ch {
        return Some((orig, ch));
    }
    if st.0 != ch {
        st.1 = 0
    }
    st.1 += 1;
    st.0 = ch.clone();
    if st.1 < 4 || ch.is_numeric() {
        Some((orig, ch))
    } else {
        Some((orig, "".into()))
    }
}

struct GraphemesPare<'a, I: 'a + Iterator<Item = (&'a str, Cow<'a, str>)>> {
    iter: I,
    graph: Option<Graphemes<'a>>,
}

impl<'a, I: 'a + Iterator<Item = (&'a str, Cow<'a, str>)>> Iterator for GraphemesPare<'a, I> {
    type Item = (&'a str, Cow<'a, str>);

    fn next(&mut self) -> Option<(&'a str, Cow<'a, str>)> {
        if let Some(ref mut gra) = self.graph {
            if let Some(x) = gra.next() {
                return Some((x, x.into()));
            }
        }
        if let Some((orig, ch)) = self.iter.next() {
            if orig != ch {
                return Some((orig, ch));
            }
            self.graph = Some(orig.graphemes(true));
            return self.next();
        };
        None
    }
}

pub fn clean_text<T: AsRef<str>>(raw: T) -> String {
    let raw = raw.as_ref();
    let mut out = String::with_capacity(raw.len());
    for x in (GraphemesPare {
            iter: regex_replacen(&Regex::new(r"\s+").unwrap(), " ", raw).into_iter(),
            graph: None,
        })
        .scan(("".into(), 0), runing_count)
        .map(|(_, x)| x) {
        out.push_str(&*x)
    }
    out.shrink_to_fit();
    out
}

fn clean_text_idx<'a, F>(raw: &'a str, len: F) -> Box<Iterator<Item = (usize, usize)> + 'a>
    where F: 'a + Fn(&str) -> usize
{
    Box::new((0..1)
        .map(|x| (x, x))
        .chain((GraphemesPare {
                iter: regex_replacen(&Regex::new(r"\s+").unwrap(), " ", raw).into_iter(),
                graph: None,
            })
            .scan(("".into(), 0), runing_count)
            .map(move |x| (len(x.0), len(&*x.1)))
            .scan((0, 0), move |st, x| {
                st.0 += x.0;
                st.1 += x.1;
                Some(*st)
            })))
}

#[allow(dead_code)]
pub fn clean_text_u8idx_in<T: AsRef<str>>(raw: T) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf8).map(|(s, _)| s).collect()
}

#[allow(dead_code)]
pub fn clean_text_u16idx_in<T: AsRef<str>>(raw: T) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf16).map(|(s, _)| s).collect()
}

#[allow(dead_code)]
pub fn clean_text_u8idx_out<T: AsRef<str>>(raw: T) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf8).map(|(_, s)| s).collect()
}

#[allow(dead_code)]
pub fn clean_text_u16idx_out<T: AsRef<str>>(raw: T) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf16).map(|(_, s)| s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wide_string::*;

    #[test]
    fn one_word() {
        assert_eq!(clean_text("Hello"), "Hello");
    }

    #[test]
    fn one_word_u8idx() {
        let text = "Hello";
        let vec_u8idx_in = clean_text_u8idx_in(text);
        let vec_u8idx_out = clean_text_u8idx_out(text);
        println!("\r\n{:?}", vec_u8idx_in);
        println!("{:?}", vec_u8idx_out);
        assert_eq!(vec_u8idx_in.len(), vec_u8idx_out.len());
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(0..5)), 0..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(0..4)), 0..4);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(4..5)), 4..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..4)), 3..4);
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
        let vec_u8idx_in = clean_text_u8idx_in(text);
        let vec_u8idx_out = clean_text_u8idx_out(text);
        println!("\r\n{:?}", vec_u8idx_in);
        println!("{:?}", vec_u8idx_out);
        assert_eq!(vec_u8idx_in.len(), vec_u8idx_out.len());
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(0..5)), 0..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..5)), 3..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(5..6)), 5..12);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(6..7)), 12..13);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..7)), 3..13);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..8)), 3..14);
    }

    #[test]
    fn two_word_with_underscore() {
        assert_eq!(clean_text("Hello _________ world!"), "Hello ___ world!");
    }

    #[test]
    fn two_word_with_underscore_u8idx() {
        let text = "Hello _________ world!";
        let vec_u8idx_in = clean_text_u8idx_in(text);
        let vec_u8idx_out = clean_text_u8idx_out(text);
        println!("\r\n{:?}", vec_u8idx_in);
        println!("{:?}", vec_u8idx_out);
        assert_eq!(vec_u8idx_in.len(), vec_u8idx_out.len());
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(0..5)), 0..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..5)), 3..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(7..9)), 7..15);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(9..10)), 9..16);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(8..12)), 8..18);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(11..15)), 17..21);
    }

    #[test]
    fn two_word_with_dash() {
        assert_eq!(clean_text("Hello ----------- world!"), "Hello --- world!");
    }

    #[test]
    fn two_word_with_dash_u8idx() {
        let text = "Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\
        \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} ----------- \u{1d565}\
        \u{1d565}\u{1d565}\u{1d565}\u{1d565}       ";
        assert_eq!(clean_text_u8idx_in(text),
                   vec![0, 1, 2, 3, 4, 5, 6, 10, 14, 18, 22, 26, 27, 28, 29, 30, 31, 32, 33, 34,
                        35, 36, 37, 38, 39, 43, 47, 51, 55, 59, 66]);
        assert_eq!(clean_text_u8idx_out(text),
                   vec![0, 1, 2, 3, 4, 5, 6, 10, 14, 18, 18, 18, 19, 20, 21, 22, 22, 22, 22, 22,
                        22, 22, 22, 22, 23, 27, 31, 35, 35, 35, 36]);
    }

    #[test]
    fn two_word_with_dash_u16idx() {
        let text = "Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\
        \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} ----------- \u{1d565}\
        \u{1d565}\u{1d565}\u{1d565}\u{1d565}       ";
        assert_eq!(clean_text_u16idx_in(text),
                   vec![0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 14, 16, 17, 18, 19, 20, 21, 22, 23, 24,
                        25, 26, 27, 28, 29, 31, 33, 35, 37, 39, 46]);
        assert_eq!(clean_text_u16idx_out(text),
                   vec![0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 12, 12, 13, 14, 15, 16, 16, 16, 16, 16,
                        16, 16, 16, 16, 17, 19, 21, 23, 23, 23, 24]);
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
        let vec_u8idx_in = clean_text_u8idx_in(text);
        let vec_u8idx_out = clean_text_u8idx_out(text);
        println!("\r\n{:?}", vec_u8idx_in);
        println!("{:?}", vec_u8idx_out);
        assert_eq!(vec_u8idx_in.len(), vec_u8idx_out.len());
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(0..5)), 0..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..5)), 3..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(5..6)), 5..6);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(6..18)), 6..26);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(18..20)), 18..28);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(14..24)), 14..32);
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
        let vec_u8idx_in = clean_text_u8idx_in(text);
        let vec_u8idx_out = clean_text_u8idx_out(text);
        println!("\r\n{:?}", vec_u8idx_in);
        println!("{:?}", vec_u8idx_out);
        assert_eq!(vec_u8idx_in.len(), vec_u8idx_out.len());
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(0..5)), 0..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(3..5)), 3..5);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(6..20)), 6..28);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(18..19)), 18..27);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(14..18)), 14..26);
        assert_eq!(invert_idx(&vec_u8idx_in, &vec_u8idx_out, &(14..22)), 14..30);
    }

    fn test_clean_text_u8idx<T: AsRef<str>>(text: T) -> bool {
        let text = text.as_ref();
        let vec_u8idx_in = clean_text_u8idx_in(text);
        let vec_u8idx_out = clean_text_u8idx_out(text);
        for (&in_idx, &out_idx) in vec_u8idx_in.iter().zip(vec_u8idx_out.iter()) {
            if clean_text(&text[..in_idx]).len() != out_idx {
                println!("\r\n{:?}", vec_u8idx_in);
                println!("{:?}", vec_u8idx_out);
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
        assert!(test_clean_text_u8idx("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\
        \u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} world!"));
    }

    #[test]
    fn quickcheck_clean_text_u8idx() {
        use quickcheck::quickcheck;
        quickcheck(test_clean_text_u8idx as fn(String) -> bool);
    }
}
