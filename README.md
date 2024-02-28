# tinyresp

[![Build Status](https://github.com/lmammino/tinyresp/actions/workflows/rust.yml/badge.svg)](https://github.com/lmammino/tinyresp/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/lmammino/tinyresp/graph/badge.svg?token=2a5OOr6Um4)](https://codecov.io/gh/lmammino/tinyresp)
[![Crates.io](https://img.shields.io/crates/v/tinyresp.svg)](https://crates.io/crates/tinyresp)
[![docs.rs](https://docs.rs/tinyresp/badge.svg)](https://docs.rs/tinyresp)

A tiny Rust library implementing the Redis Serialization Protocol (RESP)

<!-- cargo-sync-readme start -->

A simple parser for the RESP protocol (REdis Serialization Protocol).

For an overview of the procol check out the official
[Redis SErialization Protocol (RESP) documentation](https://redis.io/topics/protocol)

This library is written using [`nom`](https://crates.io/crates/nom) and therefore uses an incremental parsing approach.
This means that using the [`parse`] method will return a `Result` containing a tuple with 2 elements:
- The remaining input (which can be an empty string if the message was fully parsed)
- The parsed value (if the message was fully parsed)

# Example

```rust
use tinyresp::{parse_message, Value};

let message = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
let (remaining_input, value) = parse_message(message).unwrap();
assert_eq!(remaining_input, "");
assert_eq!(value, Value::Array(vec![
    Value::BulkString("hello"),
    Value::BulkString("world")
]));
```

<!-- cargo-sync-readme end -->

## Contributing

Everyone is very welcome to contribute to this project.
You can contribute just by submitting bugs or suggesting improvements by
[opening an issue on GitHub](https://github.com/lmammino/tinyresp/issues).


## License

Licensed under [MIT License](LICENSE). © Luciano Mammino, Roberto Gambuzzi.