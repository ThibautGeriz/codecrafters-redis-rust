use redis_starter_rust::{get_command, AsyncWriter, Command};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, Error};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let state: HashMap<String, String> = HashMap::new();
    let sharable_state = Arc::new(Mutex::new(state));
    loop {
        let (mut stream, addr) = listener.accept().await?;
        eprintln!("accepted: {addr:?}");
        let thread_sharable_state = Arc::clone(&sharable_state);
        tokio::spawn(async move {
            handle_connection(&mut stream, thread_sharable_state)
                .await
                .unwrap();
        });
    }
}

async fn handle_connection(
    stream: &mut TcpStream,
    sharable_state: Arc<Mutex<HashMap<String, String>>>,
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
                let state = sharable_state.lock().await;
                let result = (*state).get(&key);
                match result {
                    Some(value) => {
                        writer.write_simple_string(&value).await?;
                    }
                    None => {
                        writer.write_null().await?;
                    }
                }
            }
            Ok(Command::Set { key, value }) => {
                let mut state = sharable_state.lock().await;
                (*state).insert(key, value);
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
