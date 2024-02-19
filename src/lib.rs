//! A simple parser for the RESP protocol
//! Still under heavy  development
//!
//! For an overview of the procol check out the official
//! [Redis SErialization Protocol (RESP) documentation](https://redis.io/topics/protocol)

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::complete::{i32, i64, one_of},
    combinator::{eof, map, verify},
    multi::count,
    number::complete::double,
    sequence::terminated,
    IResult,
};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, PartialEq, Eq, Clone)]
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
    VerbatimString(&'a str),
    Map(HashMap<&'a str, Value<'a>>),
    Set(BTreeSet<Value<'a>>),
    Pushes,
}

pub fn parse_message(input: &str) -> IResult<&str, Value> {
    let (input, value) = terminated(parse_value, eof)(input)?;
    Ok((input, value))
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        parse_simple_string,
        parse_simple_error,
        parse_integer,
        parse_bulk_string,
        parse_array,
        parse_null,
        parse_bool,
        parse_double,
    ))(input)
}

fn u32_or_minus1(input: &str) -> IResult<&str, i32> {
    let (input, value) = verify(i32, |v| v >= &-1)(input)?;
    Ok((input, value))
}

fn crlf(input: &str) -> IResult<&str, &str> {
    tag("\r\n")(input)
}

fn parse_simple_string_raw(input: &str) -> IResult<&str, &str> {
    terminated(take_while(|c| c != '\r' && c != '\n'), crlf)(input)
}

fn parse_simple_string(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("+")(input)?;
    let (input, value) = parse_simple_string_raw(input)?;
    Ok((input, Value::SimpleString(value)))
}

fn parse_simple_error(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("-")(input)?;
    let (input, value) = parse_simple_string_raw(input)?;
    Ok((input, Value::SimpleError(value)))
}

fn parse_integer(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag(":")(input)?;
    let (input, value) = terminated(i64, crlf)(input)?;
    Ok((input, Value::Integer(value)))
}

fn parse_bulk_string(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("$")(input)?;
    let (input, length) = terminated(u32_or_minus1, crlf)(input)?;
    if length == -1 {
        return Ok((input, Value::Null));
    }

    let (input, value) = terminated(take(length as usize), crlf)(input)?;
    Ok((input, Value::BulkString(value)))
}

fn parse_array(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("*")(input)?;
    let (input, length) = terminated(u32_or_minus1, crlf)(input)?;
    if length == -1 {
        return Ok((input, Value::Null));
    }
    let (input, values) = count(parse_value, length as usize)(input)?;
    Ok((input, Value::Array(values)))
}

fn parse_null(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("_\r\n")(input)?;
    Ok((input, Value::Null))
}

fn parse_bool(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("#")(input)?;
    let (input, ch) = terminated(one_of("tf"), crlf)(input)?;
    let value = match ch {
        't' => true,
        'f' => false,
        _ => unreachable!(),
    };
    Ok((input, Value::Boolean(value)))
}

fn parse_double(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag(",")(input)?;
    let (input, value) = terminated(
        alt((
            // map(tag("inf"), || f64::INFINITY), // NOTE: This is already supported by nom
            map(tag("+inf"), |_| f64::INFINITY),
            map(tag("-inf"), |_| f64::NEG_INFINITY),
            // map(tag("nan"), || f64::NAN), // NOTE: This is already supported by nom
            double,
        )),
        crlf,
    )(input)?;

    let val_as_string = format!("{}", value);
    Ok((input, Value::Double(val_as_string)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(
            parse_message("+OK\r\n"),
            Ok(("", Value::SimpleString("OK")))
        );
        assert!(parse_message("+O\nK\r\n").is_err());
        assert!(parse_message("+OK\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_parse_simple_error() {
        assert_eq!(
            parse_message("-Error message\r\n"),
            Ok(("", Value::SimpleError("Error message")))
        );
        assert_eq!(
            parse_message("-ERR unknown command 'asdf'\r\n"),
            Ok(("", Value::SimpleError("ERR unknown command 'asdf'")))
        );
        assert_eq!(
            parse_message("-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"),
            Ok((
                "",
                Value::SimpleError(
                    "WRONGTYPE Operation against a key holding the wrong kind of value"
                )
            ))
        );
        assert!(parse_message("-Error\nmessage\r\n").is_err());
        assert!(parse_message("-Error message\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_message(":1000\r\n"), Ok(("", Value::Integer(1000))));
        assert_eq!(parse_message(":-1000\r\n"), Ok(("", Value::Integer(-1000))));
        assert!(parse_message(":1000\n").is_err());
        assert!(parse_message(":1000\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_parse_bulk_string() {
        assert_eq!(
            parse_message("$5\r\nhello\r\n"),
            Ok(("", Value::BulkString("hello")))
        );
        assert_eq!(parse_message("$0\r\n\r\n"), Ok(("", Value::BulkString(""))));
        assert_eq!(parse_message("$-1\r\n"), Ok(("", Value::Null)));
        assert_eq!(
            parse_message("$10\r\nhello\r\nfoo\r\n"),
            Ok(("", Value::BulkString("hello\r\nfoo")))
        );
        assert!(parse_message("$-2\r\n").is_err());
        assert!(parse_message("$10\r\n12345\r\n").is_err());
        assert!(parse_message("$10\r\n12345\r\n").is_err());
    }

    #[test]
    fn test_parse_array() {
        assert_eq!(parse_message("*-1\r\n"), Ok(("", Value::Null)));
        assert_eq!(parse_message("*0\r\n"), Ok(("", Value::Array(Vec::new()))));
        assert_eq!(
            parse_message("*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n"),
            Ok((
                "",
                Value::Array(vec![Value::BulkString("hello"), Value::BulkString("world")])
            ))
        );
        assert_eq!(
            parse_message("*3\r\n:1\r\n:2\r\n:3\r\n"),
            Ok((
                "",
                Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3)
                ])
            ))
        );
        assert_eq!(
            parse_message("*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$5\r\nhello\r\n"),
            Ok((
                "",
                Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                    Value::Integer(4),
                    Value::BulkString("hello")
                ])
            ))
        );
        assert_eq!(
            parse_message("*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n-World\r\n"),
            Ok((
                "",
                Value::Array(vec![
                    Value::Array(vec![
                        Value::Integer(1),
                        Value::Integer(2),
                        Value::Integer(3)
                    ]),
                    Value::Array(vec![
                        Value::SimpleString("Hello"),
                        Value::SimpleError("World")
                    ])
                ])
            ))
        );
        assert_eq!(
            parse_message("*3\r\n$5\r\nhello\r\n$-1\r\n$5\r\nworld\r\n"),
            Ok((
                "",
                Value::Array(vec![
                    Value::BulkString("hello"),
                    Value::Null,
                    Value::BulkString("world")
                ])
            ))
        );
    }

    #[test]
    fn test_null() {
        assert_eq!(parse_message("_\r\n"), Ok(("", Value::Null)));
    }

    #[test]
    fn test_bool() {
        assert_eq!(parse_message("#t\r\n"), Ok(("", Value::Boolean(true))));
        assert_eq!(parse_message("#f\r\n"), Ok(("", Value::Boolean(false))));
        assert!(parse_message("#x\r\n").is_err());
    }

    #[test]
    fn test_double() {
        assert_eq!(
            parse_message(",1.23\r\n"),
            Ok(("", Value::Double("1.23".to_string())))
        );
        assert_eq!(
            parse_message(",10\r\n"),
            Ok(("", Value::Double("10".to_string())))
        );
        assert_eq!(
            parse_message(",inf\r\n"),
            Ok(("", Value::Double("inf".to_string())))
        );
        assert_eq!(
            parse_message(",+inf\r\n"),
            Ok(("", Value::Double("inf".to_string())))
        );
        assert_eq!(
            parse_message(",-inf\r\n"),
            Ok(("", Value::Double("-inf".to_string())))
        );
        assert_eq!(
            parse_message(",nan\r\n"),
            Ok(("", Value::Double("NaN".to_string())))
        );
    }
}
