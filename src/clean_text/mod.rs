use unicode_segmentation::*;
use wide_string::*;
use std::borrow::Cow;
use regex::*;

mod regex_cleaner_pair;
pub use self::regex_cleaner_pair::*;

#[cfg(test)]
mod test;

type Pair<'a> = (&'a str, Cow<'a, str>);

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
                let s = cap.get(0).unwrap().start();
                let e = cap.get(0).unwrap().end();
                let last_match = self.last_match;
                let mut replace = String::new();
                cap.expand(self.rep, &mut replace);
                self.cap = Some((&self.text[s..e], replace.into()));
                self.last_match = e;
                Some((&self.text[last_match..s], self.text[last_match..s].into()))
            }
            None => {
                if self.last_match < self.text.len() {
                    self.last_match = self.text.len();
                    Some((&self.text[last_match..], self.text[last_match..].into()))
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
    func: F
}

impl<'a, I: 'a + Iterator<Item = Pair<'a>>, C: 'a + Iterator<Item = Pair<'a>>, F: Fn(&'a str) -> C> Iterator for FlatPair<I, C, F> {
    type Item = Pair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.current.as_mut().and_then(|cur| cur.next()){
            return Some(p);
        }
        if let Some(s) = self.source.next() {
            if s.0 != s.1 {
                return Some(s)
            } else {
                self.current = Some((self.func)(s.0));
                return self.next()
            }
        }
        None
    }
}

fn regex_replace<'r, 'a, I>(raw: I, reg: &'a RegexCleanerPair) -> Box<Iterator<Item = Pair<'a>> + 'a>
    where I: 'a + Iterator<Item = Pair<'a>>
{
    let (reg, r) = reg.to_parts();
    Box::new(FlatPair{
        source: raw,
        current: None,
        func: move |orig| RegexReplace {
                     text: orig,
                     last_match: 0,
                     captures_iter: reg.captures_iter(orig),
                     cap: None,
                     rep: r,
                 }
    })
}

fn running_count<'a>(st: &mut (&'a str, usize), ch: &'a str) -> Option<Pair<'a>> {
    if st.0 != ch {
        st.1 = 0;
        st.0 = ch.clone();
    }
    st.1 += 1;
    if st.1 < 4 || ch.chars().all(|x| x.is_numeric()) {
        Some((ch, ch.into()))
    } else {
        Some((ch, "".into()))
    }
}

fn graphemes_pair<'a, I: 'a + Iterator<Item = Pair<'a>>>(i: I) -> Box<Iterator<Item = Pair<'a>> + 'a> {
    Box::new(FlatPair{
        source: i,
        current: None,
        func: move |orig: &'a str| orig.graphemes(true).scan(("".into(), 0), running_count)
    })
}

fn trivial_pair<'a>(text: &'a str) -> Box<Iterator<Item = Pair<'a>> + 'a> {
    Box::new(Some((text, text.into())).into_iter())
}

fn clean_iter<'r: 'a, 'a>(raw: &'a str,
                          list: &'r [RegexCleanerPair])
                          -> Box<Iterator<Item = Pair<'a>> + 'a> {
    let mut out = trivial_pair(raw);
    for reg in list.iter() {
        out = regex_replace(out, reg);
    }
    Box::new(graphemes_pair(out))
}

pub fn clean_text<'r: 'a, 'a, O: ::std::iter::FromIterator<Cow<'a, str>>>(raw: &'a str, list: &'r [RegexCleanerPair]) -> O {
    clean_iter(raw, &list).map(|(_, x)| x).collect()
}

#[allow(dead_code)]
pub fn clean_text_string<T: AsRef<str>>(raw: T, list: &[RegexCleanerPair]) -> String {
    let raw = raw.as_ref();
    let mut out = String::with_capacity(raw.len());
    for (_, x) in clean_iter(raw, &list) {
        out.push_str(&*x)
    }
    out.shrink_to_fit();
    out
}

fn clean_text_idx<'r: 'a, 'a, F>(raw: &'a str,
                                 len: F,
                                 list: &'r [RegexCleanerPair])
                                 -> Box<Iterator<Item = (usize, usize)> + 'a>
    where F: 'a + Fn(&str) -> usize
{
    Box::new((0..1)
                 .map(|x| (x, x))
                 .chain(clean_iter(raw, &list)
                            .map(move |x| (len(x.0), len(&*x.1)))
                            .scan((0, 0), move |st, x| {
        st.0 += x.0;
        st.1 += x.1;
        Some(*st)
    })))
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