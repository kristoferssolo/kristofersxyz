use serde::{Deserialize, Deserializer};
use std::{fmt::Display, str::FromStr};

macro_rules! impl_deserialize_from_str {
    ($($type:ty),+ $(,)?) => {
        $(
            impl<'de> serde::Deserialize<'de> for $type {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    $crate::serde_helpers::deserialize_from_str(deserializer)
                }
            }
        )+
    };
}

pub(crate) use impl_deserialize_from_str;

pub fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}
