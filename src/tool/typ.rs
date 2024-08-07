use std::ops::{Deref, DerefMut};

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Debug, Clone)]
pub struct MyI64(i64);

impl Deref for MyI64 {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MyI64 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for MyI64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyValueVisitor;

        impl<'de> Visitor<'de> for MyValueVisitor {
            type Value = MyI64;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer or a string")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(MyI64(value))
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(MyI64(value as i64))
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(MyI64(value as i64))
            }
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(MyI64(value as i64))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let v = value.parse::<i64>().unwrap_or_default();

                Ok(MyI64(v))
            }
        }

        deserializer.deserialize_any(MyValueVisitor)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct MyDecimal(Decimal);

impl Deref for MyDecimal {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MyDecimal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for MyDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.is_empty() {
            Ok(MyDecimal(Decimal::new(0, 0)))
        } else {
            match s.parse::<Decimal>() {
                Ok(decimal) => Ok(MyDecimal(decimal)),
                Err(_) => Err(serde::de::Error::custom("Parse error")),
            }
        }
    }
}

// 将 Decimal 类型json 序列化为 i64 数值
//用法：在 字段上面加 #[serde(serialize_with = "serialize_decimal_toi64")]
fn serialize_decimal_toi64<S>(value: &Decimal, serializer: S) -> anyhow::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(value.to_i64().unwrap_or_default())
}

// 将 Decimal 类型json 序列化为 f64 数值
//用法：在 字段上面加 #[serde(serialize_with = "serialize_decimal_tof64")]
fn serialize_decimal_tof64<S>(value: &Decimal, serializer: S) -> anyhow::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(value.to_f64().unwrap_or_default())
}
