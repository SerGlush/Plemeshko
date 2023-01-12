use std::{collections::HashMap, hash::Hash};

use super::{Object, ParseError, ParseErrorKind, ParseResult, Path, Value};

pub trait FromValue: Sized {
    fn from_value(value: Value) -> ParseResult<Self>;
}

// Helpers

pub fn try_take_optional_key<T: FromValue>(
    object: &mut Object,
    key: &str,
) -> ParseResult<Option<T>> {
    match object.remove(key) {
        Some(value) => Ok(Some(T::from_value(value).map_err(|e| e.lift(key))?)),
        None => Ok(None),
    }
}

pub fn try_take_key<T: FromValue>(object: &mut Object, key: &str) -> ParseResult<T> {
    try_take_optional_key(object, key).and_then(|v| {
        v.ok_or_else(|| ParseError {
            kind: ParseErrorKind::FieldAbsent,
            path: Path::new(),
            expected: key.into(),
        })
    })
}

pub fn parse_type_err<T>() -> ParseError {
    ParseError {
        kind: ParseErrorKind::UnexpectedType,
        path: Path::new(),
        expected: std::any::type_name::<T>().into(),
    }
}

pub fn parse_type_err_res<T>() -> ParseResult<T> {
    Err(parse_type_err::<T>())
}

// Basic

impl FromValue for Value {
    fn from_value(value: Value) -> ParseResult<Self> {
        Ok(value)
    }
}

impl FromValue for () {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Null => Ok(()),
            _ => parse_type_err_res(),
        }
    }
}

impl FromValue for bool {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Bool(bool) => Ok(bool),
            _ => parse_type_err_res(),
        }
    }
}

impl FromValue for String {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::String(string) => Ok(string),
            _ => parse_type_err_res(),
        }
    }
}

// Collections

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(T::from_value(value)?)),
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Array(array) => array.into_iter().map(T::from_value).try_collect(),
            _ => parse_type_err_res(),
        }
    }
}

impl<K: From<String> + Eq + Hash, V: FromValue> FromValue for HashMap<K, V> {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Object(object) => object
                .into_iter()
                .map(|pair| Ok((K::from(pair.0), V::from_value(pair.1)?)))
                .try_collect(),
            _ => parse_type_err_res(),
        }
    }
}

impl FromValue for Object {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Object(object) => Ok(object),
            _ => parse_type_err_res(),
        }
    }
}

// Tuples

macro_rules! count_args_space_sep {
    ($name:ident) => { 1 };
    ($first:ident $($rest:ident)*) => {
        1 + count_args_space_sep!($($rest)*)
    }
}

macro_rules! tuple_from_value_impl {
    ( $head:ident $( $tail:ident )+ ) => {
        impl<$head $(,$tail)*> FromValue for ($head $(, $tail )*)
        where
            $head: FromValue,
            $( $tail: FromValue ),*
        {
            fn from_value(value: Value) -> ParseResult<Self> {
                match value {
                    Value::Array(array) => {
                        const TUPLE_LEN: usize = count_args_space_sep!($head $($tail)*);
                        if array.len() != TUPLE_LEN {
                            return Err(ParseError {
                                kind: ParseErrorKind::UnexpectedType,
                                path: Path::new(),
                                expected: format!("Array length is exactly {}.", TUPLE_LEN),
                            });
                        }
                        let mut iter = array.into_iter();
                        Ok(($head::from_value(iter.next().unwrap())? $(,$tail::from_value(iter.next().unwrap())?)*))
                    }
                    _ => parse_type_err_res(),
                }
            }
        }

        tuple_from_value_impl!($($tail)*);
    };

    ($head:ident) => {};
}

tuple_from_value_impl!(A B C D E F G H I J);

// Numeric

impl FromValue for u64 {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Number(number) => number.as_u64().ok_or_else(parse_type_err::<Self>),
            _ => parse_type_err_res(),
        }
    }
}

impl FromValue for u128 {
    fn from_value(value: Value) -> ParseResult<Self> {
        u64::from_value(value).map(u64::into)
    }
}

impl FromValue for u8 {
    fn from_value(value: Value) -> ParseResult<Self> {
        u64::from_value(value)
            .and_then(|value| value.try_into().map_err(|_| parse_type_err::<Self>()))
    }
}

impl FromValue for u16 {
    fn from_value(value: Value) -> ParseResult<Self> {
        u64::from_value(value)
            .and_then(|value| value.try_into().map_err(|_| parse_type_err::<Self>()))
    }
}

impl FromValue for u32 {
    fn from_value(value: Value) -> ParseResult<Self> {
        u64::from_value(value)
            .and_then(|value| value.try_into().map_err(|_| parse_type_err::<Self>()))
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Number(number) => number.as_i64().ok_or_else(parse_type_err::<Self>),
            _ => parse_type_err_res(),
        }
    }
}

impl FromValue for i128 {
    fn from_value(value: Value) -> ParseResult<Self> {
        i64::from_value(value).map(i64::into)
    }
}

impl FromValue for i8 {
    fn from_value(value: Value) -> ParseResult<Self> {
        i64::from_value(value)
            .and_then(|value| value.try_into().map_err(|_| parse_type_err::<Self>()))
    }
}

impl FromValue for i16 {
    fn from_value(value: Value) -> ParseResult<Self> {
        i64::from_value(value)
            .and_then(|value| value.try_into().map_err(|_| parse_type_err::<Self>()))
    }
}

impl FromValue for i32 {
    fn from_value(value: Value) -> ParseResult<Self> {
        i64::from_value(value)
            .and_then(|value| value.try_into().map_err(|_| parse_type_err::<Self>()))
    }
}

impl FromValue for f64 {
    fn from_value(value: Value) -> ParseResult<Self> {
        match value {
            Value::Number(number) => number.as_f64().ok_or_else(parse_type_err::<Self>),
            _ => parse_type_err_res(),
        }
    }
}

impl FromValue for f32 {
    fn from_value(value: Value) -> ParseResult<Self> {
        f64::from_value(value).map(|value| value as f32)
    }
}
