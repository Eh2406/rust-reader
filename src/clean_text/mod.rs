use unicode_segmentation::*;
use wide_string::*;
use std::borrow::Cow;
use regex::*;

mod regex_clener_pair;
pub use self::regex_clener_pair::*;

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

fn regex_replace<'r, 'a, I>(raw: I, reg: &'a RegexClenerPair) -> Box<Iterator<Item = Pair<'a>> + 'a>
    where I: 'a + Iterator<Item = Pair<'a>>
{
    let (reg, r) = reg.to_parts();
    Box::new(raw.flat_map(move |(orig, ch)| -> Box<Iterator<Item = Pair<'a>> + 'a> {
        if orig != ch {
            return Box::new(Some((orig, ch)).into_iter());
        }
        Box::new(RegexReplace {
                     text: orig,
                     last_match: 0,
                     captures_iter: reg.captures_iter(orig),
                     cap: None,
                     rep: r,
                 })
    }))
}

fn runing_count<'a>(st: &mut (Cow<'a, str>, usize), (orig, ch): Pair<'a>) -> Option<Pair<'a>> {
    if orig != ch {
        return Some((orig, ch));
    }
    if st.0 != ch {
        st.1 = 0
    }
    st.1 += 1;
    st.0 = ch.clone();
    if st.1 < 4 || ch.chars().all(|x| x.is_numeric()) {
        Some((orig, ch))
    } else {
        Some((orig, "".into()))
    }
}

struct GraphemesPair<'a, I: 'a + Iterator<Item = Pair<'a>>> {
    iter: I,
    graph: Option<Graphemes<'a>>,
}

impl<'a, I: 'a + Iterator<Item = Pair<'a>>> Iterator for GraphemesPair<'a, I> {
    type Item = Pair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.graph.as_mut().and_then(|gra| gra.next()) {
            return Some((x, x.into()));
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

fn graphemes_pair<'a, I: 'a + Iterator<Item = Pair<'a>>>(i: I) -> GraphemesPair<'a, I> {
    GraphemesPair {
        iter: i,
        graph: None,
    }
}

fn trivial_pair<'a>(text: &'a str) -> Box<Iterator<Item = Pair<'a>> + 'a> {
    Box::new(Some((text, text.into())).into_iter())
}

fn clean_iter<'r: 'a, 'a>(raw: &'a str,
                          list: &'r [RegexClenerPair])
                          -> Box<Iterator<Item = Pair<'a>> + 'a> {
    let mut out = trivial_pair(raw);
    for reg in list.iter() {
        out = regex_replace(out, reg);
    }
    Box::new(graphemes_pair(out).scan(("".into(), 0), runing_count))
}

pub fn clean_text<'r: 'a, 'a, O: ::std::iter::FromIterator<Cow<'a, str>>>(raw: &'a str, list: &'r [RegexClenerPair]) -> O {
    clean_iter(raw, &list).map(|(_, x)| x).collect()
}

#[allow(dead_code)]
pub fn clean_text_string<T: AsRef<str>>(raw: T, list: &[RegexClenerPair]) -> String {
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
                                 list: &'r [RegexClenerPair])
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
pub fn clean_text_u8idx_in<T: AsRef<str>>(raw: T, list: &[RegexClenerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf8, list)
        .map(|(s, _)| s)
        .collect()
}

#[allow(dead_code)]
pub fn clean_text_u16idx_in<T: AsRef<str>>(raw: T, list: &[RegexClenerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf16, list)
        .map(|(s, _)| s)
        .collect()
}

#[allow(dead_code)]
pub fn clean_text_u8idx_out<T: AsRef<str>>(raw: T, list: &[RegexClenerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf8, list)
        .map(|(_, s)| s)
        .collect()
}

#[allow(dead_code)]
pub fn clean_text_u16idx_out<T: AsRef<str>>(raw: T, list: &[RegexClenerPair]) -> Vec<usize> {
    clean_text_idx(raw.as_ref(), LenUtf::len_utf16, list)
        .map(|(_, s)| s)
        .collect()
}