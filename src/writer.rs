use tokio::{io::AsyncWriteExt, io::Error, net::TcpStream};

#[derive(Debug)]
pub struct AsyncWriter<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> AsyncWriter<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self { stream }
    }

    pub async fn write_simple_string(&mut self, print: String) -> Result<(), Error> {
        self.stream
            .write_all(format!("+{}\r\n", print).as_bytes())
            .await?;
        Ok(())
    }

    pub async fn write_error(&mut self, print: String) -> Result<(), Error> {
        self.stream
            .write_all(format!("-{}\r\n", print).as_bytes())
            .await?;
        Ok(())
    }

    pub async fn write_null(&mut self) -> Result<(), Error> {
        self.stream.write_all("$-1\r\n".as_bytes()).await?;
        Ok(())
    }
}
