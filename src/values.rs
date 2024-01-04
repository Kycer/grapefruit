use std::fmt::Debug;

use chrono::{FixedOffset, Local, Offset};

use sqlx::types::{
    chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc},
    BigDecimal,
};

use crate::GrapefruitError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ArrayType {
    Bool,
    Tinyint,
    Smallint,
    Int,
    Bigint,
    TinyUnsigned,
    SmallUnsigned,
    Unsigned,
    BigUnsigned,
    Float,
    Double,
    String,
    Char,
    Bytes,
    Json,
    ChronoDate,
    ChronoTime,
    ChronoDateTime,
    ChronoDateTimeUtc,
    ChronoDateTimeLocal,
    ChronoDateTimeWithTimeZone,
    BigDecimal,
}

impl ArrayType {
    // pub fn to_value(&self)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(Option<bool>),

    Tinyint(Option<i8>),
    Smallint(Option<i16>),
    Int(Option<i32>),
    Bigint(Option<i64>),

    TinyUnsigned(Option<u8>),
    SmallUnsigned(Option<u16>),
    Unsigned(Option<u32>),
    BigUnsigned(Option<u64>),

    Float(Option<f32>),
    Double(Option<f64>),
    BigDecimal(Option<Box<BigDecimal>>),

    Char(Option<char>),
    String(Option<String>),

    Bytes(Option<Box<Vec<u8>>>),
    Json(Option<Box<serde_json::Value>>),

    ChronoDate(Option<Box<NaiveDate>>),
    ChronoTime(Option<Box<NaiveTime>>),
    ChronoDateTime(Option<Box<NaiveDateTime>>),
    ChronoDateTimeUtc(Option<Box<DateTime<Utc>>>),
    ChronoDateTimeLocal(Option<Box<DateTime<Local>>>),
    ChronoDateTimeWithTimeZone(Option<Box<DateTime<FixedOffset>>>),

    Array(ArrayType, Option<Box<Vec<Self>>>),
}

impl Value {
    pub fn unwrap<T>(self) -> T
    where
        T: ValueType,
    {
        T::unwrap(self)
    }

    pub fn expect<T>(self, msg: &str) -> T
    where
        T: ValueType,
    {
        T::expect(self, msg)
    }

    pub fn chrono_as_naive_utc_in_string(&self) -> Option<String> {
        match self {
            Self::ChronoDate(v) => v.as_ref().map(|v| v.to_string()),
            Self::ChronoTime(v) => v.as_ref().map(|v| v.to_string()),
            Self::ChronoDateTime(v) => v.as_ref().map(|v| v.to_string()),
            Self::ChronoDateTimeUtc(v) => v.as_ref().map(|v| v.naive_utc().to_string()),
            Self::ChronoDateTimeLocal(v) => v.as_ref().map(|v| v.naive_utc().to_string()),
            Self::ChronoDateTimeWithTimeZone(v) => v.as_ref().map(|v| v.naive_utc().to_string()),
            _ => panic!("not chrono Value"),
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Value::Bool(v) => v.is_none(),
            Value::Tinyint(v) => v.is_none(),
            Value::Smallint(v) => v.is_none(),
            Value::Int(v) => v.is_none(),
            Value::Bigint(v) => v.is_none(),
            Value::TinyUnsigned(v) => v.is_none(),
            Value::SmallUnsigned(v) => v.is_none(),
            Value::Unsigned(v) => v.is_none(),
            Value::BigUnsigned(v) => v.is_none(),
            Value::Float(v) => v.is_none(),
            Value::Double(v) => v.is_none(),
            Value::BigDecimal(v) => v.is_none(),
            Value::Char(v) => v.is_none(),
            Value::String(v) => v.is_none(),
            Value::Bytes(v) => v.is_none(),
            Value::Json(v) => v.is_none(),
            Value::ChronoDate(v) => v.is_none(),
            Value::ChronoTime(v) => v.is_none(),
            Value::ChronoDateTime(v) => v.is_none(),
            Value::ChronoDateTimeUtc(v) => v.is_none(),
            Value::ChronoDateTimeLocal(v) => v.is_none(),
            Value::ChronoDateTimeWithTimeZone(v) => v.is_none(),
            Value::Array(_, v) => v.is_none(),
        }
    }

    pub fn get_str(&self) -> String {
        match self {
            Value::Char(v) => match v {
                Some(vv) => vv.to_string(),
                None => "".to_string(),
            },
            Value::String(v) => match v {
                Some(vv) => vv.clone(),
                None => "".to_string(),
            },
            _ => panic!("Unsupported type"),
        }
    }

    pub fn get_array(&self) -> Vec<Value> {
        match self {
            Value::Array(_t, v) => match v {
                Some(vv) => {
                    let mut arr = Vec::with_capacity(vv.len());
                    for value in vv.iter() {
                        arr.push(value.clone());
                    }
                    arr
                }
                None => vec![],
            },
            _ => panic!("Unsupported type"),
        }
    }
}

pub trait ValueType: Sized {
    fn try_from(v: Value) -> Result<Self, GrapefruitError>;

    fn unwrap(v: Value) -> Self {
        Self::try_from(v).unwrap()
    }

    fn expect(v: Value, msg: &str) -> Self {
        Self::try_from(v).expect(msg)
    }

    fn type_name() -> String;

    fn array_type() -> ArrayType;
}

pub trait Nullable {
    fn null() -> Value;
}

macro_rules! type_to_value {
    ( $type: ty, $name: ident ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(x))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }

        impl ValueType for $type {
            fn try_from(v: Value) -> Result<Self, GrapefruitError> {
                match v {
                    Value::$name(Some(x)) => Ok(x),
                    _ => Err(GrapefruitError::ValueTypeError()),
                }
            }

            fn type_name() -> String {
                stringify!($type).to_owned()
            }

            fn array_type() -> ArrayType {
                ArrayType::$name
            }
        }
    };
}

macro_rules! type_to_box_value {
    ( $type: ty, $name: ident ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(Box::new(x)))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }

        impl ValueType for $type {
            fn try_from(v: Value) -> Result<Self, GrapefruitError> {
                match v {
                    Value::$name(Some(x)) => Ok(*x),
                    _ => Err(GrapefruitError::ValueTypeError()),
                }
            }

            fn type_name() -> String {
                stringify!($type).to_owned()
            }

            fn array_type() -> ArrayType {
                ArrayType::$name
            }
        }
    };
}

type_to_value!(bool, Bool);
type_to_value!(i8, Tinyint);
type_to_value!(i16, Smallint);
type_to_value!(i32, Int);
type_to_value!(i64, Bigint);
type_to_value!(u8, TinyUnsigned);
type_to_value!(u16, SmallUnsigned);
type_to_value!(u32, Unsigned);
type_to_value!(u64, BigUnsigned);
type_to_value!(f32, Float);
type_to_value!(f64, Double);
type_to_value!(char, Char);
type_to_value!(String, String);

impl From<&[u8]> for Value {
    fn from(x: &[u8]) -> Value {
        Value::Bytes(Some(Box::<Vec<u8>>::new(x.into())))
    }
}

impl From<&str> for Value {
    fn from(x: &str) -> Value {
        let string: String = x.into();
        Value::String(Some(string))
    }
}

// impl From<&String> for Value {
//     fn from(x: &String) -> Value {
//         let string: String = x.into();
//         Value::String(Some(string))
//     }
// }

impl Nullable for &str {
    fn null() -> Value {
        Value::String(None)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value> + Nullable,
{
    fn from(x: Option<T>) -> Value {
        match x {
            Some(v) => v.into(),
            None => T::null(),
        }
    }
}

impl<T> From<&T> for Value
where
    T: Into<Value> + Clone,
{
    fn from(value: &T) -> Self {
        let v = value.clone().into();
        v
    }
}

impl<T> ValueType for Option<T>
where
    T: ValueType + Nullable,
{
    fn try_from(v: Value) -> Result<Self, GrapefruitError> {
        if v == T::null() {
            Ok(None)
        } else {
            Ok(Some(T::try_from(v)?))
        }
    }

    fn type_name() -> String {
        format!("Option<{}>", T::type_name())
    }

    fn array_type() -> ArrayType {
        T::array_type()
    }
}

// impl From<Cow<'_, str>> for Value {
//     fn from(x: Cow<'_, str>) -> Value {
//         x.into_owned().into()
//     }
// }

// impl ValueType for Cow<'_, str> {
//     fn try_from(v: Value) -> Result<Self, GrapefruitError> {
//         match v {
//             Value::String(Some(x)) => Ok((*x).into()),
//             _ => Err(GrapefruitError::ValueTypeError()),
//         }
//     }

//     fn type_name() -> String {
//         "Cow<str>".into()
//     }

//     fn array_type() -> ArrayType {
//         ArrayType::String
//     }
// }

type_to_box_value!(Vec<u8>, Bytes);

type_to_box_value!(serde_json::Value, Json);

type_to_box_value!(NaiveDate, ChronoDate);
type_to_box_value!(NaiveTime, ChronoTime);
type_to_box_value!(NaiveDateTime, ChronoDateTime);

impl From<DateTime<Utc>> for Value {
    fn from(v: DateTime<Utc>) -> Value {
        Value::ChronoDateTimeUtc(Some(Box::new(v)))
    }
}

impl From<DateTime<Local>> for Value {
    fn from(v: DateTime<Local>) -> Value {
        Value::ChronoDateTimeLocal(Some(Box::new(v)))
    }
}

impl From<DateTime<FixedOffset>> for Value {
    fn from(x: DateTime<FixedOffset>) -> Value {
        let v = DateTime::<FixedOffset>::from_naive_utc_and_offset(x.naive_utc(), x.offset().fix());
        Value::ChronoDateTimeWithTimeZone(Some(Box::new(v)))
    }
}

impl Nullable for DateTime<Utc> {
    fn null() -> Value {
        Value::ChronoDateTimeUtc(None)
    }
}

impl ValueType for DateTime<Utc> {
    fn try_from(v: Value) -> Result<Self, GrapefruitError> {
        match v {
            Value::ChronoDateTimeUtc(Some(x)) => Ok(*x),
            _ => Err(GrapefruitError::ValueTypeError()),
        }
    }

    fn type_name() -> String {
        stringify!(DateTime<Utc>).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::ChronoDateTimeUtc
    }
}

impl Nullable for DateTime<Local> {
    fn null() -> Value {
        Value::ChronoDateTimeLocal(None)
    }
}

impl ValueType for DateTime<Local> {
    fn try_from(v: Value) -> Result<Self, GrapefruitError> {
        match v {
            Value::ChronoDateTimeLocal(Some(x)) => Ok(*x),
            _ => Err(GrapefruitError::ValueTypeError()),
        }
    }

    fn type_name() -> String {
        stringify!(DateTime<Local>).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::ChronoDateTimeLocal
    }
}

impl Nullable for DateTime<FixedOffset> {
    fn null() -> Value {
        Value::ChronoDateTimeWithTimeZone(None)
    }
}

impl ValueType for DateTime<FixedOffset> {
    fn try_from(v: Value) -> Result<Self, GrapefruitError> {
        match v {
            Value::ChronoDateTimeWithTimeZone(Some(x)) => Ok(*x),
            _ => Err(GrapefruitError::ValueTypeError()),
        }
    }

    fn type_name() -> String {
        stringify!(DateTime<FixedOffset>).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::ChronoDateTimeWithTimeZone
    }
}

type_to_box_value!(BigDecimal, BigDecimal);

pub trait NotU8 {}
impl NotU8 for bool {}
impl NotU8 for i8 {}
impl NotU8 for i16 {}
impl NotU8 for i32 {}
impl NotU8 for i64 {}
impl NotU8 for u16 {}
impl NotU8 for u32 {}
impl NotU8 for u64 {}
impl NotU8 for f32 {}
impl NotU8 for f64 {}
impl NotU8 for char {}
impl NotU8 for String {}
impl NotU8 for Vec<u8> {}
impl NotU8 for serde_json::Value {}
impl NotU8 for NaiveDate {}
impl NotU8 for NaiveTime {}
impl NotU8 for NaiveDateTime {}
impl<Tz> NotU8 for DateTime<Tz> where Tz: chrono::TimeZone {}
impl NotU8 for BigDecimal {}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value> + NotU8 + ValueType,
{
    fn from(x: Vec<T>) -> Value {
        Value::Array(
            T::array_type(),
            Some(Box::new(x.into_iter().map(|e| e.into()).collect())),
        )
    }
}

impl<T> Nullable for Vec<T>
where
    T: Into<Value> + NotU8 + ValueType,
{
    fn null() -> Value {
        Value::Array(T::array_type(), None)
    }
}

impl<T> ValueType for Vec<T>
where
    T: ValueType + NotU8,
{
    fn try_from(v: Value) -> Result<Self, GrapefruitError> {
        match v {
            Value::Array(ty, Some(v)) if T::array_type() == ty => {
                Ok(v.into_iter().map(|e| e.unwrap()).collect())
            }
            _ => Err(GrapefruitError::ValueTypeError()),
        }
    }

    fn type_name() -> String {
        stringify!(Vec<T>).to_owned()
    }

    fn array_type() -> ArrayType {
        T::array_type()
    }
}
