use unicode_segmentation::*;

pub trait IsWhitespace {
    fn is_whitespace(&self) -> bool;
}

impl IsWhitespace for str {
    fn is_whitespace(&self) -> bool {
        self.chars().all(|x| x.is_whitespace())
    }
}

fn runing_count<'a>(st: &mut (&'a str, usize), ch: &'a str) -> Option<&'a str> {
    let c_is_whitespace = ch.is_whitespace();
    if st.0 != ch && (!st.0.is_whitespace() || !c_is_whitespace) {
        st.1 = 0
    }
    st.1 += 1;
    st.0 = ch;
    if st.1 == 1 || st.1 < 4 && !c_is_whitespace {
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

#[test]
fn one_word() {
    assert_eq!(clean_text("Hello"), "Hello");
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
fn two_word_with_underscore() {
    assert_eq!(clean_text("Hello _________ world!"), "Hello ___ world!");
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
fn two_word_with_longchar() {
    assert_eq!(clean_text("Hello \u{1d565}\u{1d565}\u{1d565}\u{1d565}\u{1d565} world!"),
               "Hello \u{1d565}\u{1d565}\u{1d565} world!");
}


#[test]
fn two_word_with_multichar() {
    assert_eq!(clean_text("Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\
                           \u{5a2} world!"),
               "Hello \u{5d4}\u{5a2}\u{5d4}\u{5a2}\u{5d4}\u{5a2} world!");
}
