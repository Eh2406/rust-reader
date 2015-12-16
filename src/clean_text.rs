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
       .fold((String::new(), '=', 0), |(mut out, last_ch, count), ch| {
           if ch != last_ch {
               out.push(ch);
               (out, ch, 1)
           } else if last_ch != ' ' && count < 3 {
               out.push(ch);
               (out, ch, count + 1)
           } else {
               (out, last_ch, count + 1)
           }
       })
       .0
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
