use redis_starter_rust::{get_command, Command};

use tokio::io::{AsyncReadExt, AsyncWriteExt, Error};
use tokio::net::{TcpListener, TcpStream};

const PONG: &[u8] = b"+PONG\r\n";

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
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
            Ok(Command::Ping) => {
                stream.write_all(PONG).await?;
            }
            Ok(Command::Echo { print }) => {
                stream
                    .write_all(format!("+{}\r\n", print).as_bytes())
                    .await?;
            }
            Ok(Command::Unknown) => {
                stream.write_all(b"-Unknown command\r\n").await?;
            }
            Err(_) => {
                stream.write_all(b"-Unknown error\r\n").await?;
            }
        }
    }
    Ok(())
}
