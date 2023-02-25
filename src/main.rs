use redis_starter_rust::{get_command, Command};

use tokio::io::{AsyncReadExt, AsyncWriteExt, Error};
use tokio::net::{TcpListener, TcpStream};

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
        let command = get_command(&buf);
        match command {
            Some(Command::Ping) => {
                stream.write_all(PONG).await?;
            }
            Some(Command::Echo { print }) => {
                stream
                    .write_all(format!("+{}\r\n", print).as_bytes())
                    .await?;
            }
            _ => {
                stream.write_all(b"-Unknow command\r\n").await?;
            }
        }
    }
    Ok(())
}
