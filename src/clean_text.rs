// TODO: switch to https://unicode-rs.github.io/unicode-segmentation/

fn only_spaces(ch: char) -> char {
    if ch.is_whitespace() {
        ' '
    } else {
        ch
    }
}

fn runing_count(st: &mut (char, usize), ch: char) -> Option<bool> {
    if st.0 != ch && (!st.0.is_whitespace() || !ch.is_whitespace()) {
        st.1 = 0
    }
    st.1 += 1;
    st.0 = ch;
    Some(st.1 == 1 || st.1 < 4 && !ch.is_whitespace())
}

pub fn clean_text<T: AsRef<str>>(raw: T) -> String {
    let raw = raw.as_ref();
    raw.chars()
       .zip(raw.chars().scan(('\x00', 0), runing_count))
       .filter(|&(_, test)| test)
       .map(|(ch, _)| ch)
       .map(only_spaces)
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
