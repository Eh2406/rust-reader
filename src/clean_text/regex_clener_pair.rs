use regex::*;
use serde::{Serialize, Serializer, Deserializer};
use serde::de::{self, Deserialize, Visitor, SeqVisitor};

#[derive(Debug)]
pub struct RegexClenerPair {
    regex: Regex,
    rep: String,
}

impl RegexClenerPair {
    fn new<T: AsRef<str>>(regex: T, rep: String) -> Result<RegexClenerPair, Error> {
        Ok(RegexClenerPair {
               regex: Regex::new(regex.as_ref())?,
               rep: rep,
           })
    }
    pub fn prep_list(input: &[(&str, &str)]) -> Result<Vec<RegexClenerPair>, Error> {
        input
            .iter()
            .map(|&(ref reg, rep)| RegexClenerPair::new(reg, rep.to_string()))
            .collect()
    }
    pub fn to_parts(&self) -> (&Regex, &str) {
        let &RegexClenerPair {
                    regex: ref reg,
                    rep: ref r,
                } = self;
        (reg, r)
    }
}

impl Serialize for RegexClenerPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq_fixed_size(2)?;
        seq.serialize_element(self.regex.as_str())?;
        seq.serialize_element(&self.rep)?;
        seq.end()
    }
}

impl Deserialize for RegexClenerPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct RegexClenerPairVisitor;

        impl Visitor for RegexClenerPairVisitor {
            type Value = RegexClenerPair;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a pair for of regex and replacement")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<RegexClenerPair, V::Error>
                where V: SeqVisitor
            {
                let regex: String = match visitor.visit()? {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(0, &self));
                    }
                };
                let rep: String = match visitor.visit()? {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(1, &self));
                    }
                };
                Ok(RegexClenerPair {
                       regex: Regex::new(&regex)
                           .map_err(|_| {
                                        de::Error::invalid_value(de::Unexpected::Str(&regex), &self)
                                    })?,
                       rep: rep,
                   })
            }
        }

        const FIELDS: &'static [&'static str] = &["regex", "rep"];
        deserializer.deserialize_struct("RegexClenerPair", FIELDS, RegexClenerPairVisitor)
    }
}