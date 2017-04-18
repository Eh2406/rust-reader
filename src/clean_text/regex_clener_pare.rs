use regex::*;
use serde::{Serialize, Serializer, Deserializer};
use serde::de::{self, Deserialize, Visitor, SeqVisitor};

#[derive(Debug)]
pub struct RegexClenerPare {
    regex: Regex,
    rep: String,
}

impl RegexClenerPare {
    fn new<T: AsRef<str>>(regex: T, rep: String) -> Result<RegexClenerPare, Error> {
        Ok(RegexClenerPare {
               regex: Regex::new(regex.as_ref())?,
               rep: rep,
           })
    }
    pub fn prep_list(input: &[(&str, &str)]) -> Result<Vec<RegexClenerPare>, Error> {
        input
            .iter()
            .map(|&(ref reg, rep)| RegexClenerPare::new(reg, rep.to_string()))
            .collect()
    }
    pub fn to_parts(&self) -> (&Regex, &str) {
        let &RegexClenerPare {
                    regex: ref reg,
                    rep: ref r,
                } = self;
        (reg, r)
    }
}

impl Serialize for RegexClenerPare {
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

impl Deserialize for RegexClenerPare {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct RegexClenerPareVisitor;

        impl Visitor for RegexClenerPareVisitor {
            type Value = RegexClenerPare;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a pair for of regex and replacement")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<RegexClenerPare, V::Error>
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
                Ok(RegexClenerPare {
                       regex: Regex::new(&regex)
                           .map_err(|_| {
                                        de::Error::invalid_value(de::Unexpected::Str(&regex), &self)
                                    })?,
                       rep: rep,
                   })
            }
        }

        const FIELDS: &'static [&'static str] = &["regex", "rep"];
        deserializer.deserialize_struct("RegexClenerPare", FIELDS, RegexClenerPareVisitor)
    }
}