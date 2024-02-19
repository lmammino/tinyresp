//! A simple parser for the RESP protocol
//! Still under heavy  development
//!
//! For an overview of the procol check out the official
//! [Redis SErialization Protocol (RESP) documentation](https://redis.io/topics/protocol)

use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::{i32, i64, one_of},
    combinator::{eof, verify},
    sequence::terminated,
    IResult,
};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Value<'a> {
    SimpleString(&'a str),
    SimpleError(&'a str),
    Integer(i64),
    BulkString(&'a str),
    Array(Vec<Value<'a>>),
    Null,
    Boolean(bool),
    Double(&'a str),
    BigNumber(&'a str),
    BulkError(&'a str),
    VerbatimString(&'a str),
    // Map(HashMap<&'a str, Value<'a>>),
    // Set(HashSet<Value<'a>>),
    Pushes,
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    let (input, type_char) = one_of("+-:$*_#,(!=%~>")(input)?;
    let parser = match type_char {
        '+' => parse_simple_string,
        '-' => parse_simple_error,
        ':' => parse_integer,
        '$' => parse_bulk_string,
        '*' => todo!(),
        '_' => todo!(),
        '#' => todo!(),
        ',' => todo!(),
        '(' => todo!(),
        '!' => todo!(),
        '=' => todo!(),
        '%' => todo!(),
        '~' => todo!(),
        '>' => todo!(),
        _ => unreachable!("Invalid type char"),
    };

    terminated(parser, eof)(input)
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
    let (input, value) = parse_simple_string_raw(input)?;
    Ok((input, Value::SimpleString(value)))
}

fn parse_simple_error(input: &str) -> IResult<&str, Value> {
    let (input, value) = parse_simple_string_raw(input)?;
    Ok((input, Value::SimpleError(value)))
}

fn parse_integer(input: &str) -> IResult<&str, Value> {
    let (input, value) = terminated(i64, crlf)(input)?;
    Ok((input, Value::Integer(value)))
}

fn parse_bulk_string(input: &str) -> IResult<&str, Value> {
    let (input, length) = terminated(u32_or_minus1, crlf)(input)?;
    if length == -1 {
        return Ok((input, Value::Null));
    }

    let (input, value) = terminated(take(length as usize), crlf)(input)?;
    Ok((input, Value::BulkString(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(parse_value("+OK\r\n"), Ok(("", Value::SimpleString("OK"))));
        assert!(parse_value("+O\nK\r\n").is_err());
        assert!(parse_value("+OK\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_parse_simple_error() {
        assert_eq!(
            parse_value("-Error message\r\n"),
            Ok(("", Value::SimpleError("Error message")))
        );
        assert_eq!(
            parse_value("-ERR unknown command 'asdf'\r\n"),
            Ok(("", Value::SimpleError("ERR unknown command 'asdf'")))
        );
        assert_eq!(
            parse_value("-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"),
            Ok((
                "",
                Value::SimpleError(
                    "WRONGTYPE Operation against a key holding the wrong kind of value"
                )
            ))
        );
        assert!(parse_value("-Error\nmessage\r\n").is_err());
        assert!(parse_value("-Error message\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_value(":1000\r\n"), Ok(("", Value::Integer(1000))));
        assert_eq!(parse_value(":-1000\r\n"), Ok(("", Value::Integer(-1000))));
        assert!(parse_value(":1000\n").is_err());
        assert!(parse_value(":1000\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_parse_bulk_string() {
        assert_eq!(
            parse_value("$5\r\nhello\r\n"),
            Ok(("", Value::BulkString("hello")))
        );
        assert_eq!(parse_value("$0\r\n\r\n"), Ok(("", Value::BulkString(""))));
        assert_eq!(parse_value("$-1\r\n"), Ok(("", Value::Null)));
        assert_eq!(
            parse_value("$10\r\nhello\r\nfoo\r\n"),
            Ok(("", Value::BulkString("hello\r\nfoo")))
        );
        assert!(parse_value("$-2\r\n").is_err());
        assert!(parse_value("$10\r\n12345\r\n").is_err());
        assert!(parse_value("$10\r\n12345\r\n").is_err());
    }
}
