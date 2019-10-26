pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

#[macro_export]
macro_rules! serialize_with_display {
    ($type:ty) => {
        impl ::serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                $crate::serde::string::serialize(self, serializer)
            }
        }
    };
}

#[macro_export]
macro_rules! deserialize_with_parse {
    ($type:ty) => {
        impl<'de> ::serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                $crate::serde::string::deserialize(deserializer)
            }
        }
    };
}

pub mod u64_or_string {
    use serde::{de, Deserializer};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = u64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a non-negative integer or a string")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(v)
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::u64_or_string;
    use serde_derive::Deserialize;
    #[test]
    fn u64_or_string() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Foo {
            #[serde(with = "u64_or_string")]
            bar: u64,
        }
        assert_eq!(
            serde_json::from_str::<Foo>(r#"{"bar": 42}"#).unwrap(),
            Foo { bar: 42 }
        );
        assert_eq!(
            serde_json::from_str::<Foo>(r#"{"bar": "42"}"#).unwrap(),
            Foo { bar: 42 }
        )
    }
}
