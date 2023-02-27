use redis_starter_rust::{get_command, AsyncWriter, Command};

use tokio::io::{AsyncReadExt, Error};
use tokio::net::{TcpListener, TcpStream};

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
        let mut writer = AsyncWriter::new(stream);
        match command {
            Ok(Command::Ping) => {
                writer.write_simple_string(String::from("PONG")).await?;
            }
            Ok(Command::Echo { print }) => {
                writer.write_simple_string(print).await?;
            }
            Ok(Command::Get { key: _ }) => {
                writer
                    .write_error(String::from("not yet implemented"))
                    .await?;
            }
            Ok(Command::Set { key: _, value: _ }) => {
                writer
                    .write_error(String::from("not yet implemented"))
                    .await?;
            }
            Ok(Command::Unknown) => {
                writer.write_error(String::from("Unknown command")).await?;
            }
            Err(_) => {
                writer.write_error(String::from("Unknown error")).await?;
            }
        }
    }
    Ok(())
}
