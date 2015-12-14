pub fn clean_text(raw: &str) -> String {
    let mut out = String::new();
    let mut last_ch = '=';
    let mut count = 0;
    for ch in raw.chars() {
        if ch.is_whitespace() {
            if !last_ch.is_whitespace() {
                count = 1;
                last_ch = ' ';
                out.push(' ');
            }
        } else if ch != last_ch {
            count = 1;
            last_ch = ch;
            out.push(ch);
        } else if count < 3 {
            count += 1;
            out.push(ch);
        }
    }
    out
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
