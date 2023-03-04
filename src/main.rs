use redis_starter_rust::{get_command, AsyncWriter, Command, Store};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, Error};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let store = Store::new();
    let sharable_store = Arc::new(Mutex::new(store));
    loop {
        let (mut stream, addr) = listener.accept().await?;
        eprintln!("accepted: {addr:?}");
        let thread_sharable_store = Arc::clone(&sharable_store);
        tokio::spawn(async move {
            handle_connection(&mut stream, thread_sharable_store)
                .await
                .unwrap();
        });
    }
}

async fn handle_connection(
    stream: &mut TcpStream,
    sharable_store: Arc<Mutex<Store>>,
) -> std::result::Result<(), Error> {
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
                writer.write_simple_string(&(String::from("PONG"))).await?;
            }
            Ok(Command::Echo { print }) => {
                writer.write_simple_string(&print).await?;
            }
            Ok(Command::Get { key }) => {
                let mut store = sharable_store.lock().await;
                let result = (*store).get(key);
                match result {
                    Some(value) => {
                        writer.write_simple_string(&value).await?;
                    }
                    None => {
                        writer.write_null().await?;
                    }
                }
            }
            Ok(Command::Set { key, value, expiry }) => {
                let mut store = sharable_store.lock().await;
                (*store).set(key, value, expiry);
                writer.write_simple_string(&(String::from("OK"))).await?;
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
