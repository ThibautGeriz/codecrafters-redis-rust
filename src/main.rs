use tokio::io::{AsyncWriteExt, Error, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};

const PONG: &[u8] = "+PONG\r\n".as_bytes();

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let(mut stream, addr) = listener.accept().await?;
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

        stream.write(PONG).await?;
    }

    Ok(())
}

