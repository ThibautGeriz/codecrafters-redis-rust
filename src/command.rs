use std::io::Cursor;
use std::io::Seek;
use std::time::Duration;

use crate::parser::parse;
use crate::parser::parse_int;
use crate::parser::ParsingError;
use crate::parser::Value;

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping,
    Echo {
        print: String,
    },
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
        expiry: Option<Duration>,
    },
    Unknown,
}

const START_OF_COMMAND: u8 = b'*';

pub fn get_command(buffer: &[u8]) -> Result<Command, ParsingError> {
    let mut cursor = Cursor::new(buffer);
    // let t = cursor.seek(std::io::SeekFrom::Start(0))?;
    if buffer.is_empty() || buffer[0] != START_OF_COMMAND {
        return Err(ParsingError::FormatError);
    }
    cursor.seek(std::io::SeekFrom::Start(1))?;
    let mut line_count = match parse_int(buffer, &mut cursor) {
        Ok(Value::Int { value }) => value,
        _ => 0,
    };
    let mut parsed_lines: Vec<Value> = vec![];
    while line_count > 0 {
        let parsed_line = parse(buffer, &mut cursor)?;
        parsed_lines.push(parsed_line);

        line_count -= 1;
    }
    match parsed_lines.get(0) {
        Some(Value::String { value }) => {
            if value == &String::from("ping") {
                return Ok(Command::Ping);
            } else if value == &String::from("echo") {
                match parsed_lines.get(1) {
                    Some(Value::String { value }) => {
                        return Ok(Command::Echo {
                            print: String::from(value),
                        });
                    }
                    _ => return Err(ParsingError::FormatError),
                }
            } else if value == &String::from("get") {
                match parsed_lines.get(1) {
                    Some(Value::String { value }) => {
                        return Ok(Command::Get {
                            key: String::from(value),
                        });
                    }
                    _ => return Err(ParsingError::FormatError),
                }
            } else if value == &String::from("set") {
                match (parsed_lines.get(1), parsed_lines.get(2)) {
                    (Some(Value::String { value: key }), Some(Value::String { value })) => {
                        let mut expiry: Option<u64> = None;
                        if Some(&Value::String {
                            value: String::from("px"),
                        }) == parsed_lines.get(3)
                        {
                            if let Some(Value::String { value: _expiry }) = parsed_lines.get(4) {
                                expiry = _expiry.parse::<u64>().ok();
                            }
                        }
                        return Ok(Command::Set {
                            key: String::from(key),
                            value: String::from(value),
                            expiry: expiry.map(Duration::from_millis),
                        });
                    }
                    _ => {
                        return Err(ParsingError::FormatError);
                    }
                }
            }
            Ok(Command::Unknown)
        }
        _ => Err(ParsingError::FormatError),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::command::{get_command, Command};
    #[test]
    fn parse_input_return_unknown_if_empty() {
        let result = get_command(&[]);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn parse_input_return_ping() {
        let input = "*1\r\n$4\r\nping\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), Command::Ping);
    }

    #[test]
    fn parse_input_return_echo() {
        let input = "*2\r\n$4\r\necho\r\n$8\r\ntititoto\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap(),
            Command::Echo {
                print: String::from("tititoto")
            }
        );
    }

    #[test]
    fn parse_input_return_get() {
        let input = "*2\r\n$3\r\nget\r\n$8\r\ntititoto\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap(),
            Command::Get {
                key: String::from("tititoto")
            }
        );
    }

    #[test]
    fn parse_input_return_set() {
        let input = "*3\r\n$3\r\nset\r\n$4\r\ntiti\r\n$4\r\ntoto\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap(),
            Command::Set {
                key: String::from("titi"),
                value: String::from("toto"),
                expiry: None
            }
        );
    }

    #[test]
    fn parse_input_return_set_with_expiry() {
        let input =
            "*5\r\n$3\r\nset\r\n$4\r\ntiti\r\n$4\r\ntoto\r\n$2\r\npx\r\n$4\r\n1234\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap(),
            Command::Set {
                key: String::from("titi"),
                value: String::from("toto"),
                expiry: Some(Duration::from_millis(1234))
            }
        );
    }
}
