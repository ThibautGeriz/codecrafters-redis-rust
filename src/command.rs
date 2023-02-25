use std::io::Cursor;
use std::io::Seek;

use crate::parser::parse;
use crate::parser::parse_int;
use crate::parser::Value;

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping,
    Echo { print: String },
    Unknown,
}

const START_OF_COMMAND: u8 = b'*';

pub fn get_command(buffer: &[u8]) -> Option<Command> {
    let mut cursor = Cursor::new(buffer);
    // let t = cursor.seek(std::io::SeekFrom::Start(0))?;
    if buffer.is_empty() || buffer[0] != START_OF_COMMAND {
        return None;
    }
    cursor.seek(std::io::SeekFrom::Start(1)).unwrap();
    let mut line_count = match parse_int(buffer, &mut cursor) {
        Some(Value::Int { value }) => value,
        _ => 0,
    };
    let mut parsed_lines: Vec<Value> = vec![];
    while line_count > 0 {
        if let Some(parsed_line) = parse(buffer, &mut cursor) {
            parsed_lines.push(parsed_line);
        }
        line_count -= 1;
    }
    match parsed_lines.get(0) {
        Some(Value::String { value }) => {
            if value == &String::from("ping") {
                return Some(Command::Ping);
            } else if value == &String::from("echo") {
                match parsed_lines.get(1) {
                    Some(Value::String { value }) => {
                        return Some(Command::Echo {
                            print: String::from(value),
                        });
                    }
                    _ => return None,
                }
            }
            Some(Command::Unknown)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use crate::command::{get_command, Command};
    #[test]
    fn parse_input_return_unknown_if_empty() {
        let result = get_command(&[]);
        assert_eq!(result.is_none(), true);
    }

    #[test]
    fn parse_input_return_ping() {
        let input = "*1\r\n$4\r\nping\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap(), Command::Ping);
    }

    #[test]
    fn parse_input_return_echo() {
        let input = "*2\r\n$4\r\necho\r\n$8\r\ntititoto\r\n".as_bytes();
        let result = get_command(input);
        assert_eq!(result.is_some(), true);
        assert_eq!(
            result.unwrap(),
            Command::Echo {
                print: String::from("tititoto")
            }
        );
    }
}
