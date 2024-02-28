//! A simple parser for the RESP protocol (REdis Serialization Protocol).
//!
//! For an overview of the procol check out the official
//! [Redis SErialization Protocol (RESP) documentation](https://redis.io/topics/protocol)
//!
//! This library is written using [`nom`](https://crates.io/crates/nom) and therefore uses an incremental parsing approach.
//! This means that using the [`parse`] method will return a `Result` containing a tuple with 2 elements:
//! - The remaining input (which can be an empty string if the message was fully parsed)
//! - The parsed value (if the message was fully parsed)
//!
//! # Example
//!
//! ```
//! use tinyresp::{parse_message, Value};
//!
//! let message = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
//! let (remaining_input, value) = parse_message(message).unwrap();
//! assert_eq!(remaining_input, "");
//! assert_eq!(value, Value::Array(vec![
//!     Value::BulkString("hello"),
//!     Value::BulkString("world")
//! ]));
//! ```

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::complete::{digit1, i64, one_of, u32},
    combinator::{eof, map, opt},
    multi::count,
    number::complete::double,
    sequence::terminated,
    IResult,
};

mod value;
pub use value::*;

/// Parses a RESP message using an incremental parsing approach.
/// This means that the parser will return a `Result` containing a tuple with 2 elements:
/// - The remaining input (which can be an empty string if the message was fully parsed)
/// - The parsed value (if the message was fully parsed)
///
/// # Example
///
/// ```
/// use tinyresp::{parse_message, Value};
///
/// let message = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
/// let (remaining_input, value) = parse_message(message).unwrap();
/// assert_eq!(remaining_input, "");
/// assert_eq!(value, Value::Array(vec![
///     Value::BulkString("hello"),
///     Value::BulkString("world")
/// ]));
/// ```
pub fn parse_message(input: &str) -> IResult<&str, Value> {
    let (input, value) = terminated(parse_value, eof)(input)?;
    Ok((input, value))
}

/// Parses a RESP message and returns the parsed value.
/// This is a convenience function that uses `parse_message` internally and returns the parsed value.
/// To get better error reporting and be able to check if there's unparsed data, use [`parse_message`] instead.
pub fn parse(input: &str) -> Result<Value, String> {
    let (_, value) = parse_message(input).map_err(|e| format!("{}", e))?;
    Ok(value)
}

fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        parse_simple_string,
        parse_simple_error,
        parse_integer,
        parse_bulk_string,
        parse_array,
        parse_null,
        parse_bool,
        parse_double,
        parse_bignumber,
        parse_bulk_error,
        parse_verbatim_string,
        parse_map,
        parse_set,
        parse_pushes,
    ))(input)
}

fn values_sequence(input: &str, multiplier: usize) -> IResult<&str, Vec<Value>> {
    let (input, length) = terminated(u32, crlf)(input)?;
    let (input, values) = count(parse_value, length as usize * multiplier)(input)?;
    Ok((input, values))
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

fn parse_bulk_string_raw(input: &str) -> IResult<&str, &str> {
    let (input, length) = terminated(u32, crlf)(input)?;
    let (input, value) = terminated(take(length as usize), crlf)(input)?;
    Ok((input, value))
}

fn parse_bulk_string(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("$")(input)?;
    let (input, value) = parse_bulk_string_raw(input)?;
    Ok((input, Value::BulkString(value)))
}

fn parse_bulk_error(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("!")(input)?;
    let (input, value) = parse_bulk_string_raw(input)?;
    Ok((input, Value::BulkError(value)))
}

fn parse_array(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("*")(input)?;
    let (input, values) = values_sequence(input, 1)?;
    Ok((input, Value::Array(values)))
}

fn parse_null(input: &str) -> IResult<&str, Value> {
    let (input, _) = alt((tag("$-1\r\n"), tag("*-1\r\n"), tag("_\r\n")))(input)?;
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

fn plus_or_minus(input: &str) -> IResult<&str, char> {
    one_of("+-")(input)
}

fn parse_bignumber(input: &str) -> IResult<&str, Value> {
    let original_input = input;
    let (input, _) = tag("(")(input)?;
    let (input, sign) = opt(plus_or_minus)(input)?;
    let (input, digits) = terminated(digit1, crlf)(input)?;
    let num_slice = &original_input[1..digits.len() + if sign.is_some() { 2 } else { 1 }];
    Ok((input, Value::BigNumber(num_slice)))
}

fn parse_verbatim_string(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("=")(input)?;
    let (input, length) = terminated(u32, crlf)(input)?;
    let (input, encoding) = terminated(take(3usize), tag(":"))(input)?;
    let (input, value) = terminated(take(length as usize - 4usize), crlf)(input)?;
    Ok((input, Value::VerbatimString(encoding, value)))
}

fn parse_map(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("%")(input)?;
    let (input, keys_and_values) = values_sequence(input, 2)?;

    let (keys, values) = keys_and_values.into_iter().enumerate().fold(
        (Vec::new(), Vec::new()),
        |(mut keys, mut values), (idx, val)| {
            if idx % 2 == 0 {
                keys.push(val);
            } else {
                values.push(val);
            }

            (keys, values)
        },
    );

    Ok((input, Value::Map(keys, values)))
}

fn parse_set(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag("~")(input)?;
    let (input, values) = values_sequence(input, 1)?;
    Ok((input, Value::Set(values.into_iter().collect())))
}

fn parse_pushes(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag(">")(input)?;
    let (input, values) = values_sequence(input, 1)?;
    Ok((input, Value::Pushes(values)))
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
    fn test_parse_bulk_error() {
        assert!(parse_message("!-1\r\n").is_err());

        assert_eq!(
            parse_message("!21\r\nSYNTAX invalid syntax\r\n"),
            Ok(("", Value::BulkError("SYNTAX invalid syntax")))
        );

        assert_eq!(
            parse_message("!5\r\nhello\r\n"),
            Ok(("", Value::BulkError("hello")))
        );
        assert_eq!(parse_message("!0\r\n\r\n"), Ok(("", Value::BulkError(""))));

        assert_eq!(
            parse_message("!10\r\nhello\r\nfoo\r\n"),
            Ok(("", Value::BulkError("hello\r\nfoo")))
        );
        assert!(parse_message("!-2\r\n").is_err());
        assert!(parse_message("!10\r\n12345\r\n").is_err());
        assert!(parse_message("!10\r\n12345\r\n").is_err());
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

    #[test]
    fn test_bignumber() {
        assert_eq!(
            parse_message("(3492890328409238509324850943850943825024385\r\n"),
            Ok((
                "",
                Value::BigNumber("3492890328409238509324850943850943825024385")
            ))
        );
        assert_eq!(
            parse_message("(+3492890328409238509324850943850943825024385\r\n"),
            Ok((
                "",
                Value::BigNumber("+3492890328409238509324850943850943825024385")
            ))
        );
        assert_eq!(
            parse_message("(-3492890328409238509324850943850943825024385\r\n"),
            Ok((
                "",
                Value::BigNumber("-3492890328409238509324850943850943825024385")
            ))
        );
        assert!(parse_message("(+1234-1234\r\n").is_err());
    }

    #[test]
    fn test_verbatim_string() {
        assert_eq!(
            parse_message("=15\r\ntxt:Some string\r\n"),
            Ok(("", Value::VerbatimString("txt", "Some string")))
        );
        assert_eq!(
            parse_message("=5\r\ntxt:1\r\n"),
            Ok(("", Value::VerbatimString("txt", "1")))
        );
        assert_eq!(
            parse_message("=5\r\nraw:1\r\n"),
            Ok(("", Value::VerbatimString("raw", "1")))
        );
        assert!(parse_message("=5\r\nraw:1\r\nTHIS_SHOULD_NOT_BE_HERE").is_err());
    }

    #[test]
    fn test_map() {
        assert_eq!(
            parse_message("%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n"),
            Ok((
                "",
                Value::Map(
                    vec![Value::SimpleString("first"), Value::SimpleString("second")],
                    vec![Value::Integer(1), Value::Integer(2)]
                )
            ))
        );
    }

    #[test]
    fn test_set() {
        assert_eq!(
            parse_message("~3\r\n:1\r\n:2\r\n:3\r\n"),
            Ok((
                "",
                Value::Set(
                    vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]
                        .into_iter()
                        .collect()
                )
            ))
        );
    }

    #[test]
    fn test_pushes() {
        assert_eq!(
            parse_message(">3\r\n:1\r\n:2\r\n:3\r\n"),
            Ok((
                "",
                Value::Pushes(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3)
                ])
            ))
        );
    }

    #[test]
    fn test_parse() {
        let message = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let value = parse(message).unwrap();
        assert_eq!(
            value,
            Value::Array(vec![Value::BulkString("hello"), Value::BulkString("world")])
        );

        let message = "BOGUS";
        assert!(parse(message).is_err());
    }
}
