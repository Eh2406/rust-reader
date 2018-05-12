use regex::*;
use serde::de::{self, Deserialize, SeqAccess, Visitor};
use serde::{Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct RegexCleanerPair {
    regex: Regex,
    rep: String,
}

impl RegexCleanerPair {
    pub fn new<T: AsRef<str>>(regex: T, rep: String) -> Result<RegexCleanerPair, Error> {
        Ok(RegexCleanerPair {
            regex: Regex::new(regex.as_ref())?,
            rep,
        })
    }
    pub fn prep_list(input: &[(&str, &str)]) -> Result<Vec<RegexCleanerPair>, Error> {
        input
            .iter()
            .map(|&(reg, rep)| RegexCleanerPair::new(reg, rep.to_string()))
            .collect()
    }
    pub fn to_parts(&self) -> (&Regex, &str) {
        let &RegexCleanerPair {
            regex: ref reg,
            rep: ref r,
        } = self;
        (reg, r)
    }
}

impl Serialize for RegexCleanerPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(self.regex.as_str())?;
        seq.serialize_element(&self.rep)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for RegexCleanerPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RegexCleanerPairVisitor;

        impl<'de> Visitor<'de> for RegexCleanerPairVisitor {
            type Value = RegexCleanerPair;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a pair for of regex and replacement")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<RegexCleanerPair, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let regex: String = match visitor.next_element()? {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(0, &self));
                    }
                };
                let rep: String = match visitor.next_element()? {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(1, &self));
                    }
                };
                Ok(RegexCleanerPair {
                    regex: Regex::new(&regex)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&regex), &self))?,
                    rep,
                })
            }
        }

        const FIELDS: &[&str] = &["regex", "rep"];
        deserializer.deserialize_struct("RegexCleanerPair", FIELDS, RegexCleanerPairVisitor)
    }
}
