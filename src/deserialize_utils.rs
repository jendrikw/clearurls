use alloc::borrow::Cow;
use alloc::fmt;
use alloc::vec::Vec;
use core::marker::PhantomData;

use regex::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};
use serde::de::{Error as _, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

/// Deserialize a [`Regex`]
/// The result will have the `case_insensitive` flag set.
pub(crate) fn deserialize_regex<'de, D>(d: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <Cow<'_, str>>::deserialize(d)?;
    RegexBuilder::new(&s)
        .case_insensitive(true)
        .build()
        .map_err(D::Error::custom)
}

/// Deserialize a [`Vec<Regex>`].
/// All regexes will have the `case_insensitive` flag set.
pub(crate) fn deserialize_regex_vec<'de, D>(d: D) -> Result<Vec<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RegexVecVisitor;
    impl<'a> Visitor<'a> for RegexVecVisitor {
        type Value = Vec<Regex>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("valid sequence")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'a>,
        {
            let cap = seq.size_hint().unwrap_or(0);
            let mut vec = Vec::with_capacity(cap);
            while let Some(el) = seq.next_element::<Cow<'_, str>>()? {
                let regex = RegexBuilder::new(&el)
                    .case_insensitive(true)
                    .build()
                    .map_err(A::Error::custom)?;
                vec.push(regex);
            }
            Ok(vec)
        }
    }

    d.deserialize_seq(RegexVecVisitor)
}

/// Deserialize a [`RegexSet`].
/// All regexes will have the `case_insensitive` flag set.
pub(crate) fn deserialize_regex_set<'de, D>(d: D) -> Result<RegexSet, D::Error>
where
    D: Deserializer<'de>,
{
    let regexes = <Vec<Cow<'_, str>>>::deserialize(d)?;
    RegexSetBuilder::new(regexes)
        .case_insensitive(true)
        .build()
        .map_err(D::Error::custom)
}

/// Deserialize a [`Vec`] from a map by ignoring the keys.
pub(crate) fn deserialize_map_as_vec<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct MapAsVecVisitor<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for MapAsVecVisitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("valid map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let capacity = map.size_hint().unwrap_or(0);
            let mut vec = Vec::with_capacity(capacity);
            while let Some((_, v)) = map.next_entry::<IgnoredAny, T>()? {
                vec.push(v);
            }
            Ok(vec)
        }
    }

    d.deserialize_map(MapAsVecVisitor(PhantomData))
}

#[cfg(test)]
mod tests {
    use crate::deserialize_utils::*;
    use serde_json::error::Category;
    use serde_json::json;

    #[test]
    fn test_deserialize_regex() {
        let regex = deserialize_regex(json!("a")).unwrap();
        assert!(regex.is_match("A"));
        let error = deserialize_regex(json!("[")).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_regex(json!(true)).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_regex(json!([])).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
    }

    #[test]
    fn test_deserialize_regex_set_error() {
        let error = deserialize_regex_set(json!([".*", 1])).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_regex_set(json!("")).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_regex_set(json!(["["])).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
    }

    #[test]
    fn test_deserialize_regex_vec_error() {
        let error = deserialize_regex_vec(json!([".*", 1])).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_regex_vec(json!("")).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_regex_vec(json!(["["])).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
    }

    #[test]
    fn test_deserialize_map_as_vec_error() {
        let error = deserialize_map_as_vec::<_, bool>(json!(true)).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
        let error = deserialize_map_as_vec::<_, bool>(json!({"a": 5})).unwrap_err();
        assert_eq!(error.classify(), Category::Data);
    }
}
