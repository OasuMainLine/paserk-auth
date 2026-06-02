use std::ops::Deref;

use serde::{Deserialize, Serialize, de::Visitor};

/// A field that could be either a i32 number or a string
#[derive(Serialize, Clone)]
pub struct FlexibleNumber(pub i32);

impl Deref for FlexibleNumber {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for FlexibleNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FlexibleNumberVisitor;
        impl<'de> Visitor<'de> for FlexibleNumberVisitor {
            type Value = FlexibleNumber;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("i32 number or string")
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(FlexibleNumber(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let number = v.trim().parse::<i32>().map_err(serde::de::Error::custom)?;

                Ok(FlexibleNumber(number))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.to_string())
            }
        }

        deserializer.deserialize_any(FlexibleNumberVisitor)
    }
}

impl Into<FlexibleNumber> for i32 {
    fn into(self) -> FlexibleNumber {
        FlexibleNumber(self)
    }
}
