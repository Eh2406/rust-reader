fn only_spaces(ch: char) -> char {
    if ch.is_whitespace() {
        ' '
    } else {
        ch
    }
}

pub fn clean_text(raw: &str) -> String {
    raw.chars()
       .map(only_spaces)
       .scan(('\x00', 0), |st, ch| {
           if st.0 != ch {
               st.1 = 0
           } else {
               st.1 += 1
           };
           st.0 = ch;
           Some(*st)
       })
       .filter(|&(ch, count)| !(count >= 1 && ch.is_whitespace()))
       .filter(|&(_, count)| !(count >= 3))
       .map(|(ch, _)| ch)
       .collect()
}

#[test]
fn one_word() {
    assert_eq!(clean_text("Hello"), "Hello");
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
