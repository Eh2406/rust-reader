extern crate test;
use self::test::Bencher;
use super::*;
use super::test::RE_LIST;

#[bench]
fn short_text(b: &mut Bencher) {
    print!("{:}", clean_text_string("", &RE_LIST));
    b.iter(|| clean_text_string("Hello", &RE_LIST));
}

#[bench]
fn long_text(b: &mut Bencher) {
    print!("{:}", clean_text_string("", &RE_LIST));
    b.iter(|| clean_text_string(include_str!("p&p.txt"), &RE_LIST));
}
