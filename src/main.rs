use std::io::prelude::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::net::TcpListener;

const PONG: &[u8] = "+PONG\r\n".as_bytes();

fn main() -> std::result::Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    loop {
        let (stream, addr) = listener.accept()?;
        eprintln!("accepted: {addr:?}");
        handle_one_stream(stream);
    }
}

fn handle_one_stream(stream: std::net::TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    loop {
        match reader.fill_buf() {
            Ok(buf) => {
                if buf.is_empty() {
                    return;
                }
                if let Err(e) = writer.write_all(PONG) {
                    eprintln!("failed to write_all: {e:?}");
                    return;
                }
                if let Err(e) = writer.flush() {
                    eprint!("failed to flush: {e:?}");
                    return;
                }
                let len = buf.len();
                reader.consume(len);
            }
            Err(e) => {
                eprintln!("failed to fill_buf: {e:?}");
                return;
            }
        }
    }
}
