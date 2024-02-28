use std::collections::{BTreeSet, HashMap};
use thiserror::Error;

/// Represents a RESP value
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Value<'a> {
    SimpleString(&'a str),
    SimpleError(&'a str),
    Integer(i64),
    BulkString(&'a str),
    Array(Vec<Value<'a>>),
    Null,
    Boolean(bool),
    Double(String),
    BigNumber(&'a str),
    BulkError(&'a str),
    VerbatimString(&'a str, &'a str),
    /// Maps are represented as a tuple of two vectors, the first one contains the keys and the second one the values
    Map(Vec<Value<'a>>, Vec<Value<'a>>),
    Set(BTreeSet<Value<'a>>),
    Pushes(Vec<Value<'a>>),
}

/// Represents an error that can occur when trying to convert a [Value] to a HashMap
#[derive(Error, Debug)]
pub enum ToHashMapError<'a> {
    #[error("The current value is not a map")]
    NotAMap,
    #[error("A given key is not a string: {0:?}")]
    KeyNotString(&'a Value<'a>),
}

impl<'a> Value<'a> {
    /// Helper method to check if the current value is a [Value::SimpleString]
    pub fn is_simple_string(&self) -> bool {
        matches!(self, Value::SimpleString(_))
    }

    /// Helper method to check if the current value is a [Value::SimpleError]
    pub fn is_simple_error(&self) -> bool {
        matches!(self, Value::SimpleError(_))
    }

    /// Helper method to check if the current value is a [Value::Integer]
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// Helper method to check if the current value is a [Value::BulkString]
    pub fn is_bulk_string(&self) -> bool {
        matches!(self, Value::BulkString(_))
    }

    /// Helper method to check if the current value is a [Value::Array]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Helper method to check if the current value is [Value::Null]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Helper method to check if the current value is a [Value::Boolean]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Helper method to check if the current value is a [Value::Double]
    pub fn is_double(&self) -> bool {
        matches!(self, Value::Double(_))
    }

    /// Helper method to check if the current value is a [Value::BigNumber]
    pub fn is_bignumber(&self) -> bool {
        matches!(self, Value::BigNumber(_))
    }

    /// Helper method to check if the current value is a [Value::BulkError]
    pub fn is_bulk_error(&self) -> bool {
        matches!(self, Value::BulkError(_))
    }

    /// Helper method to check if the current value is a [Value::VerbatimString]
    pub fn is_verbatim_string(&self) -> bool {
        matches!(self, Value::VerbatimString(_, _))
    }

    /// Helper method to check if the current value is a [Value::Map]
    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_, _))
    }

    /// Helper method to check if the current value is a [Value::Set]
    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(_))
    }

    /// Helper method to check if the current value is a [Value::Pushes]
    pub fn is_pushes(&self) -> bool {
        matches!(self, Value::Pushes(_))
    }

    /// Helper method to check if the current value can be converted to a string.
    /// This will be `true` for [Value::SimpleString], [Value::SimpleError], [Value::BulkString],
    /// [Value::BulkError], [Value::Double], [Value::BigNumber], and [Value::VerbatimString].
    pub fn is_string_like(&self) -> bool {
        self.is_simple_string()
            || self.is_simple_error()
            || self.is_bulk_string()
            || self.is_bulk_error()
            || self.is_double()
            || self.is_bignumber()
            || self.is_verbatim_string()
    }

    /// Helper method to check if the current value can be converted to an array.
    /// This will be `true` for [Value::Array] and [Value::Pushes].
    pub fn is_array_like(&self) -> bool {
        self.is_array() || self.is_pushes()
    }

    /// Helper method that tries to get a string reference from the current value.
    /// This will return `Some(&str)` for [Value::SimpleString], [Value::SimpleError], [Value::BulkString],
    /// [Value::BulkError], [Value::Double], [Value::BigNumber], and [Value::VerbatimString].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::SimpleString(s) => Some(s),
            Value::SimpleError(s) => Some(s),
            Value::BulkString(s) => Some(s),
            Value::Double(s) => Some(s),
            Value::BigNumber(s) => Some(s),
            Value::BulkError(s) => Some(s),
            Value::VerbatimString(_, s) => Some(s),
            _ => None,
        }
    }

    /// Helper method that tries to get an integer from the current value.
    /// This will return `Some(i64)` for [Value::Integer].
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Helper method that tries to get a double from the current value.
    /// This will return `Some(f64)` for [Value::Double].
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Double(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Helper method that tries to get a boolean from the current value.
    /// This will return `Some(bool)` for [Value::Boolean].
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Helper method that tries to get an array reference from the current value.
    /// This will return `Some(&Vec<Value>)` for [Value::Array] and [Value::Pushes].
    pub fn as_array(&self) -> Option<&Vec<Value<'a>>> {
        match self {
            Value::Array(a) => Some(a),
            Value::Pushes(p) => Some(p),
            _ => None,
        }
    }

    /// Helper method that tries to get a map reference from the current value.
    /// This will return `Some((&Vec<Value>, &Vec<Value>)` for [Value::Map].
    pub fn as_map(&self) -> Option<(&Vec<Value<'a>>, &Vec<Value<'a>>)> {
        match self {
            Value::Map(k, v) => Some((k, v)),
            _ => None,
        }
    }

    /// Helper method that tries to get a set reference from the current value.
    /// This will return `Some(&BTreeSet<Value>)` for [Value::Set].
    pub fn as_set(&self) -> Option<&BTreeSet<Value<'a>>> {
        match self {
            Value::Set(s) => Some(s),
            _ => None,
        }
    }

    /// Helper method that tries to convert a [Value::Map] to an HashMap.
    /// This conversion will succeed only if the current variant is a [Value::Map] and all the keys are strings.
    pub fn try_to_hashmap(&self) -> Result<HashMap<String, &Value<'_>>, ToHashMapError<'_>> {
        match self {
            Value::Map(keys, values) => {
                let mut map = HashMap::new();
                for (key, value) in keys.iter().zip(values.iter()) {
                    if key.is_string_like() {
                        map.insert(key.as_str().unwrap().to_string(), value);
                    } else {
                        return Err(ToHashMapError::KeyNotString(key));
                    }
                }
                Ok(map)
            }
            _ => Err(ToHashMapError::NotAMap),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_simple_string() {
        let value = Value::SimpleString("hello");
        assert!(value.is_simple_string());

        // not a simple string
        let value = Value::SimpleError("hello");
        assert!(!value.is_simple_string());
    }

    #[test]
    fn test_is_simple_error() {
        let value = Value::SimpleError("hello");
        assert!(value.is_simple_error());

        // not a simple error
        let value = Value::SimpleString("hello");
        assert!(!value.is_simple_error());
    }

    #[test]
    fn test_is_integer() {
        let value = Value::Integer(42);
        assert!(value.is_integer());

        // not an integer
        let value = Value::SimpleString("hello");
        assert!(!value.is_integer());
    }

    #[test]
    fn test_is_bulk_string() {
        let value = Value::BulkString("hello");
        assert!(value.is_bulk_string());

        // not a bulk string
        let value = Value::SimpleString("hello");
        assert!(!value.is_bulk_string());
    }

    #[test]
    fn test_is_array() {
        let value = Value::Array(vec![]);
        assert!(value.is_array());

        // not an array
        let value = Value::SimpleString("hello");
        assert!(!value.is_array());
    }

    #[test]
    fn test_is_null() {
        let value = Value::Null;
        assert!(value.is_null());

        // not null
        let value = Value::SimpleString("hello");
        assert!(!value.is_null());
    }

    #[test]
    fn test_is_bool() {
        let value = Value::Boolean(true);
        assert!(value.is_bool());

        // not a boolean
        let value = Value::SimpleString("hello");
        assert!(!value.is_bool());
    }

    #[test]
    fn test_is_double() {
        let value = Value::Double("3.14".to_string());
        assert!(value.is_double());

        // not a double
        let value = Value::SimpleString("hello");
        assert!(!value.is_double());
    }

    #[test]
    fn test_is_bignumber() {
        let value = Value::BigNumber("1234567890");
        assert!(value.is_bignumber());

        // not a bignumber
        let value = Value::Array(vec![]);
        assert!(!value.is_bignumber());
    }

    #[test]
    fn test_is_bulk_error() {
        let value = Value::BulkError("hello");
        assert!(value.is_bulk_error());

        // not a bulk error
        let value = Value::Array(vec![]);
        assert!(!value.is_bulk_error());
    }

    #[test]
    fn test_is_verbatim_string() {
        let value = Value::VerbatimString("txt", "hello");
        assert!(value.is_verbatim_string());

        // not a verbatim string
        let value = Value::Array(vec![]);
        assert!(!value.is_verbatim_string());
    }

    #[test]
    fn test_is_map() {
        let value = Value::Map(vec![], vec![]);
        assert!(value.is_map());

        // not a map
        let value = Value::Array(vec![]);
        assert!(!value.is_map());
    }

    #[test]
    fn test_is_set() {
        let value = Value::Set(BTreeSet::new());
        assert!(value.is_set());

        // not a set
        let value = Value::Array(vec![]);
        assert!(!value.is_set());
    }

    #[test]
    fn test_is_pushes() {
        let value = Value::Pushes(vec![]);
        assert!(value.is_pushes());

        // not a pushes
        let value = Value::SimpleString("hello");
        assert!(!value.is_pushes());
    }

    #[test]
    fn test_is_string_like() {
        let value = Value::SimpleString("hello");
        assert!(value.is_string_like());

        let value = Value::SimpleError("hello");
        assert!(value.is_string_like());

        let value = Value::BulkString("hello");
        assert!(value.is_string_like());

        let value = Value::BulkError("hello");
        assert!(value.is_string_like());

        let value = Value::Double("3.14".to_string());
        assert!(value.is_string_like());

        let value = Value::BigNumber("1234567890");
        assert!(value.is_string_like());

        let value = Value::VerbatimString("txt", "hello");
        assert!(value.is_string_like());

        // not a string-like
        let value = Value::Array(vec![]);
        assert!(!value.is_string_like());
    }

    #[test]
    fn test_is_array_like() {
        let value = Value::Array(vec![]);
        assert!(value.is_array_like());

        let value = Value::Pushes(vec![]);
        assert!(value.is_array_like());

        // not an array-like
        let value = Value::SimpleString("hello");
        assert!(!value.is_array_like());
    }

    #[test]
    fn test_as_str() {
        let value = Value::SimpleString("hello");
        assert_eq!(value.as_str(), Some("hello"));

        let value = Value::SimpleError("hello");
        assert_eq!(value.as_str(), Some("hello"));

        let value = Value::BulkString("hello");
        assert_eq!(value.as_str(), Some("hello"));

        let value = Value::Double("3.14".to_string());
        assert_eq!(value.as_str(), Some("3.14"));

        let value = Value::BigNumber("1234567890");
        assert_eq!(value.as_str(), Some("1234567890"));

        let value = Value::BulkError("hello");
        assert_eq!(value.as_str(), Some("hello"));

        let value = Value::VerbatimString("txt", "hello");
        assert_eq!(value.as_str(), Some("hello"));

        // not a string-like
        let value = Value::Array(vec![]);
        assert_eq!(value.as_str(), None);
    }

    #[test]
    fn test_as_i64() {
        let value = Value::Integer(42);
        assert_eq!(value.as_i64(), Some(42));

        // not an integer
        let value = Value::SimpleString("hello");
        assert_eq!(value.as_i64(), None);
    }

    #[test]
    fn test_as_f64() {
        let value = Value::Double(std::f64::consts::PI.to_string());
        assert_eq!(value.as_f64(), Some(std::f64::consts::PI));

        // not a double
        let value = Value::SimpleString("hello");
        assert_eq!(value.as_f64(), None);
    }

    #[test]
    fn test_as_bool() {
        let value = Value::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));

        // not a boolean
        let value = Value::SimpleString("hello");
        assert_eq!(value.as_bool(), None);
    }

    #[test]
    fn test_as_array() {
        let value = Value::Array(vec![]);
        assert_eq!(value.as_array(), Some(&vec![]));

        let value = Value::Pushes(vec![]);
        assert_eq!(value.as_array(), Some(&vec![]));

        // not an array
        let value = Value::SimpleString("hello");
        assert_eq!(value.as_array(), None);
    }

    #[test]
    fn test_as_map() {
        let value = Value::Map(vec![], vec![]);
        assert_eq!(value.as_map(), Some((&vec![], &vec![])));

        // not a map
        let value = Value::Array(vec![]);
        assert_eq!(value.as_map(), None);
    }

    #[test]
    fn test_as_set() {
        let value = Value::Set(BTreeSet::new());
        assert_eq!(value.as_set(), Some(&BTreeSet::new()));

        // not a set
        let value = Value::Array(vec![]);
        assert_eq!(value.as_set(), None);
    }

    #[test]
    fn test_try_to_hashmap() {
        let value = Value::Map(
            vec![
                Value::SimpleString("key1"),
                Value::SimpleString("key2"),
                Value::SimpleString("key3"),
            ],
            vec![
                Value::SimpleString("value1"),
                Value::SimpleString("value2"),
                Value::SimpleString("value3"),
            ],
        );
        let map = value.try_to_hashmap().unwrap();
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("key1").unwrap().as_str(), Some("value1"));
        assert_eq!(map.get("key2").unwrap().as_str(), Some("value2"));
        assert_eq!(map.get("key3").unwrap().as_str(), Some("value3"));

        // not a map
        let value = Value::SimpleString("hello");
        assert!(matches!(
            value.try_to_hashmap(),
            Err(ToHashMapError::NotAMap)
        ));

        // key is not a string-like
        let value = Value::Map(
            vec![Value::Array(vec![])],
            vec![Value::SimpleString("value")],
        );
        assert!(matches!(
            value.try_to_hashmap(),
            Err(ToHashMapError::KeyNotString(_))
        ));
    }
}
