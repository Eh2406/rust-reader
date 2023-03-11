use regex::*;
use std::borrow::Cow;
use unicode_segmentation::*;
use crate::wide_string::*;

mod regex_cleaner_pair;
pub use self::regex_cleaner_pair::*;

// // un comment and add #![feature(test)] to main to benchmark
// #[cfg(test)]
// mod bench;

#[cfg(test)]
mod test;

type Pair<'a> = (&'a str, Option<Cow<'a, str>>);

struct RegexReplace<'r, 'a> {
    text: &'a str,
    last_match: usize,
    captures_iter: CaptureMatches<'r, 'a>,
    cap: Option<Pair<'a>>,
    rep: &'r str,
}

impl<'r, 'a> Iterator for RegexReplace<'r, 'a> {
    type Item = Pair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let last_match = self.last_match;
        if let Some(cap) = self.cap.take() {
            return Some(cap);
        }
        match self.captures_iter.next() {
            Some(cap) => {
                // unwrap on 0 is OK because captures only reports matches
                let cap0 = cap.get(0).unwrap();
                let unmatched = &self.text[self.last_match..cap0.start()];
                let mut replace = String::new();
                cap.expand(self.rep, &mut replace);
                self.cap = Some((cap0.as_str(), Some(replace.into())));
                self.last_match = cap0.end();
                Some((unmatched, None))
            }
            None => {
                if self.last_match < self.text.len() {
                    self.last_match = self.text.len();
                    Some((&self.text[last_match..], None))
                } else {
                    None
                }
            }
        }
    }
}

struct RegexSubstitute<'r, 'a> {
    text: &'a str,
    last_match: usize,
    captures_iter: Matches<'r, 'a>,
    cap: Option<Pair<'a>>,
    rep: &'a str,
}

impl<'r, 'a> Iterator for RegexSubstitute<'r, 'a> {
    type Item = Pair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let last_match = self.last_match;
        if let Some(cap) = self.cap.take() {
            return Some(cap);
        }
        match self.captures_iter.next() {
            Some(cap) => {
                let unmatched = &self.text[self.last_match..cap.start()];
                self.cap = Some((cap.as_str(), Some(self.rep.into())));
                self.last_match = cap.end();
                Some((unmatched, None))
            }
            None => {
                if self.last_match < self.text.len() {
                    self.last_match = self.text.len();
                    Some((&self.text[last_match..], None))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug)]
struct FlatPair<I, C, F> {
    source: I,
    current: Option<C>,
    func: F,
}

impl<I, C, F> FlatPair<I, C, F> {
    fn new_box<'a>(i: I, f: F) -> Box<FlatPair<I, C, F>>
    where
        I: 'a + Iterator<Item = Pair<'a>>,
        C: 'a + Iterator<Item = Pair<'a>>,
        F: Fn(&'a str) -> C,
    {
        Box::new(FlatPair {
            source: i,
            current: None,
            func: f,
        })
    }
}

impl<'a, I, C, F> Iterator for FlatPair<I, C, F>
where
    I: 'a + Iterator<Item = Pair<'a>>,
    C: 'a + Iterator<Item = Pair<'a>>,
    F: Fn(&'a str) -> C,
{
    type Item = Pair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.current.as_mut().and_then(|cur| cur.next()) {
            return Some(p);
        }
        if let Some(s) = self.source.next() {
            if s.1.is_some() {
                return Some(s);
            } else {
                self.current = Some((self.func)(s.0));
                return self.next();
            }
        }
        None
    }
}

fn regex_replace<'r, 'a, I>(
    raw: I,
    reg: &'a RegexCleanerPair,
) -> Box<dyn Iterator<Item = Pair<'a>> + 'a>
where
    I: 'a + Iterator<Item = Pair<'a>>,
{
    let (reg, mut r) = reg.to_parts();
    if r.no_expansion().is_some() {
        FlatPair::new_box(raw, move |orig| RegexSubstitute {
            text: orig,
            last_match: 0,
            captures_iter: reg.find_iter(orig),
            cap: None,
            rep: r,
        })
    } else {
        FlatPair::new_box(raw, move |orig| RegexReplace {
            text: orig,
            last_match: 0,
            captures_iter: reg.captures_iter(orig),
            cap: None,
            rep: r,
        })
    }
}

fn running_count<'a>(st: &mut (&'a str, usize), ch: &'a str) -> Option<Pair<'a>> {
    if st.0 != ch {
        st.1 = 0;
        st.0 = ch;
    }
    st.1 += 1;
    Some((
        ch,
        if st.1 < 4 || ch.chars().all(|x| x.is_numeric()) {
            None
        } else {
            Some("".into())
        },
    ))
}

fn graphemes_pair<'a, I: 'a + Iterator<Item = Pair<'a>>>(
    i: I,
) -> Box<dyn Iterator<Item = Pair<'a>> + 'a> {
    FlatPair::new_box(i, move |orig: &'a str| {
        orig.graphemes(true).scan(("", 0), running_count)
    })
}

fn trivial_pair<'a>(text: &'a str) -> Box<dyn Iterator<Item = Pair<'a>> + 'a> {
    Box::new(Some((text, None)).into_iter())
}

fn clean_iter<'r: 'a, 'a>(
    raw: &'a str,
    list: &'r [RegexCleanerPair],
) -> Box<dyn Iterator<Item = Pair<'a>> + 'a> {
    let mut out = trivial_pair(raw);
    for reg in list.iter() {
        out = regex_replace(out, reg);
    }
    Box::new(graphemes_pair(out))
}

pub fn clean_text<'r: 'a, 'a, O>(raw: &'a str, list: &'r [RegexCleanerPair]) -> O
where
    O: ::std::iter::FromIterator<Cow<'a, str>>,
{
    clean_iter(raw, list)
        .map(|(o, r)| r.unwrap_or_else(|| o.into()))
        .collect()
}

fn clean_text_idx<'r: 'a, 'a, F>(
    raw: &'a str,
    len: F,
    list: &'r [RegexCleanerPair],
) -> Box<dyn Iterator<Item = (usize, usize)> + 'a>
where
    F: 'a + Fn(&str) -> usize,
{
    Box::new(
        (0..1).map(|x| (x, x)).chain(
            clean_iter(raw, list)
                .map(move |(o, r)| (len(o), len(&*r.unwrap_or_else(|| o.into()))))
                .scan((0, 0), move |st, x| {
                    st.0 += x.0;
                    st.1 += x.1;
                    Some(*st)
                }),
        ),
    )
}

#[allow(dead_code)]
pub fn clean_text_u8idx_in<T: AsRef<str>>(raw: T, list: &[RegexCleanerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf8, list)
        .map(|(s, _)| s)
        .collect()
}

#[allow(dead_code)]
pub fn clean_text_u16idx_in<T: AsRef<str>>(raw: T, list: &[RegexCleanerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf16, list)
        .map(|(s, _)| s)
        .collect()
}

#[allow(dead_code)]
pub fn clean_text_u8idx_out<T: AsRef<str>>(raw: T, list: &[RegexCleanerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf8, list)
        .map(|(_, s)| s)
        .collect()
}

#[allow(dead_code)]
pub fn clean_text_u16idx_out<T: AsRef<str>>(raw: T, list: &[RegexCleanerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf16, list)
        .map(|(_, s)| s)
        .collect()
}
