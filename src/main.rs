use std::str::from_utf8;

use tokio::io::{AsyncReadExt, AsyncWriteExt, Error};
use tokio::net::{TcpListener, TcpStream};

const START_OF_COMMAND: u8 = b'*';
const END_OF_LINE: u8 = b'\r';
const PONG: &[u8] = b"+PONG\r\n";

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (mut stream, addr) = listener.accept().await?;
        eprintln!("accepted: {addr:?}");
        tokio::spawn(async move {
            handle_connection(&mut stream).await.unwrap();
        });
    }
}

async fn handle_connection(stream: &mut TcpStream) -> std::result::Result<(), Error> {
    let mut buf = [0; 512];

    loop {
        let bytes_read = stream.read(&mut buf).await?;
        if bytes_read == 0 {
            println!("client closed the connection");
            break;
        }
        if buf[0] != START_OF_COMMAND {
            stream
                .write(b"-instruction do not start with *\r\n")
                .await?;
            eprintln!("instruction do not start with *");
            break;
        }
        let (end_index, command_length_start) = get_end_of_line_index(&buf, 0);
        let _line_count = from_utf8(&buf[1..end_index])
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let (command_length_end, command_start) = get_end_of_line_index(&buf, command_length_start);
        let _command_length = from_utf8(&buf[(command_length_start + 1)..command_length_end])
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let (command_end, first_arg_length_start) = get_end_of_line_index(&buf, command_start);
        let command = from_utf8(&buf[(command_start)..command_end]).unwrap();
        if command == "ping" {
            stream.write(PONG).await?;
        } else if command == "echo" {
            let (_, first_arg_start) = get_end_of_line_index(&buf, first_arg_length_start);
            let (first_arg_end, _) = get_end_of_line_index(&buf, first_arg_start);
            let first_arg = from_utf8(&buf[(first_arg_start)..first_arg_end]).unwrap();

            stream
                .write(format!("+{}\r\n", first_arg).as_bytes())
                .await?;
        } else {
            stream.write(b"-Unknow command\r\n").await?;
            eprintln!("unknown command {}", command)
        }
    }

    Ok(())
}

fn get_end_of_line_index(buf: &[u8; 512], start: usize) -> (usize, usize) {
    let mut i = start;
    loop {
        i += 1;
        if buf[i] == END_OF_LINE {
            break;
        }
    }
    return (i, i + 2);
}
