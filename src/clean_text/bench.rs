extern crate test;
use self::test::Bencher;
use super::*;
use super::test::{RE_LIST, clean_text_string};

#[bench]
fn short_text(b: &mut Bencher) {
    print!("{:}", clean_text_string("", &RE_LIST));
    b.iter(|| clean_text_string("Hello", &RE_LIST));
}

#[bench]
fn half_pap_text(b: &mut Bencher) {
    // if we maintain O(n) the should take half the time
    print!("{:}", clean_text_string("", &RE_LIST));
    let pap = include_str!("p&p.txt");
    b.iter(|| clean_text_string(&pap[..(pap.len() / 2)], &RE_LIST));
}

#[bench]
fn pap_text(b: &mut Bencher) {
    print!("{:}", clean_text_string("", &RE_LIST));
    let pap = include_str!("p&p.txt");
    b.iter(|| clean_text_string(pap, &RE_LIST));
}

#[bench]
fn pap_wide(b: &mut Bencher) {
    print!("{:}", clean_text_string("", &RE_LIST));
    let pap = include_str!("p&p.txt");
    b.iter(|| clean_text::<WideString>(pap, &RE_LIST));
}
