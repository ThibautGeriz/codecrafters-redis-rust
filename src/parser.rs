use std::io::Cursor;
use std::io::Seek;
use std::str::from_utf8;

#[derive(Debug, PartialEq)]
pub enum Value {
    Int { value: i32 },
    String { value: String },
    Array { items: Vec<Value> },
    Null,
}

pub fn parse(buffer: &[u8], cursor: &mut Cursor<&[u8]>) -> Option<Value> {
    match buffer[cursor.position() as usize] {
        b':' => {
            cursor.seek(std::io::SeekFrom::Current(1)).unwrap();
            parse_int(buffer, cursor)
        }
        b'$' => {
            cursor.seek(std::io::SeekFrom::Current(1)).unwrap();
            parse_bulk_string(buffer, cursor)
        }
        b'*' => {
            cursor.seek(std::io::SeekFrom::Current(1)).unwrap();
            parse_array(buffer, cursor)
        }
        _ => None,
    }
}

pub fn parse_int(buffer: &[u8], cursor: &mut Cursor<&[u8]>) -> Option<Value> {
    let mut step: u64 = 1;
    let position = cursor.position() as usize;
    while buffer[position + step as usize + 1] != b'\n' {
        step += 1;
    }
    let int_slice = &buffer[position..position + step as usize];
    let value = from_utf8(int_slice)
        .ok()
        .and_then(|s| s.parse::<i32>().ok());
    cursor
        .seek(std::io::SeekFrom::Current((step as i64) + 2))
        .unwrap();
    value.map(|v| Value::Int { value: v })
}

fn parse_bulk_string(buffer: &[u8], cursor: &mut Cursor<&[u8]>) -> Option<Value> {
    match parse_int(buffer, cursor) {
        Some(Value::Int { value }) => {
            if value == -1 {
                cursor
                    .seek(std::io::SeekFrom::Current((value as i64) + 3))
                    .unwrap();
                return Some(Value::Null);
            }
            let position = cursor.position() as usize;
            let string_slice = &buffer[position..position + value as usize];
            cursor
                .seek(std::io::SeekFrom::Current((value as i64) + 2))
                .unwrap();
            from_utf8(string_slice).ok().map(|s| Value::String {
                value: String::from(s),
            })
        }
        _ => None,
    }
}

fn parse_array(buffer: &[u8], cursor: &mut Cursor<&[u8]>) -> Option<Value> {
    match parse_int(buffer, cursor) {
        Some(Value::Int { value: 0 }) => Some(Value::Array { items: vec![] }),
        Some(Value::Int { value }) => {
            let mut items: Vec<Value> = vec![];
            let mut item_count_left = value;
            while item_count_left > 0 {
                item_count_left -= 1;
                items.push(parse(buffer, cursor).unwrap());
            }
            Some(Value::Array { items })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use crate::parser::{parse, parse_int, Value};

    #[test]
    fn parse_int_one_digit() {
        let input = "OOPS1\r\nOOPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse_int(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap(), Value::Int { value: 1 });
        assert_eq!(cursor.position(), 7);
    }

    #[test]
    fn parse_int_three_digit() {
        let input = "OOPS123\r\nOOPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse_int(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap(), Value::Int { value: 123 });
        assert_eq!(cursor.position(), 9);
    }

    #[test]
    fn parse_return_int() {
        let input = "OOPS:123\r\nOOPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap(), Value::Int { value: 123 });
        assert_eq!(cursor.position(), 10);
    }

    #[test]
    fn parse_return_string() {
        let input = "OOPS$4\r\necho\r\nOOPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(
            result.unwrap(),
            Value::String {
                value: String::from("echo")
            }
        );
        assert_eq!(cursor.position(), 14);
    }
    #[test]
    fn parse_return_empty_string() {
        let input = "OOPS$0\r\n\r\nOOPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(
            result.unwrap(),
            Value::String {
                value: String::from("")
            }
        );
        assert_eq!(cursor.position(), 10);
    }

    #[test]
    fn parse_return_null() {
        let input = "OOPS$-1\r\n\r\nOOPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap(), Value::Null);
        assert_eq!(cursor.position(), 11);
        assert_eq!(input[cursor.position() as usize], b'O');
    }

    #[test]
    fn parse_return_empty_array() {
        let input = "OUPS*0\r\nOUPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(result.unwrap(), Value::Array { items: vec![] });
        assert_eq!(cursor.position(), 8);
    }

    #[test]
    fn parse_return_array_of_2_string() {
        let input = "OUPS*2\r\n$5\r\nhello\r\n$5\r\nworld\r\nOUPS".as_bytes();
        let mut cursor = Cursor::new(input);
        cursor.seek(std::io::SeekFrom::Start(4)).unwrap();
        let result = parse(input, &mut cursor);
        assert_eq!(result.is_some(), true);
        assert_eq!(
            result.unwrap(),
            Value::Array {
                items: vec![
                    Value::String {
                        value: String::from("hello")
                    },
                    Value::String {
                        value: String::from("world")
                    }
                ]
            }
        );
        assert_eq!(cursor.position(), 30);
        assert_eq!(input[cursor.position() as usize], b'O');
    }
}
