use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait Connection {
    fn set_ip(&mut self, ip: String);
    fn set_port(&mut self, port: u16);
    fn get_ip(&self, ) -> String;
    fn get_port(&self) -> u16;
    async fn open(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn close(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
    async fn receive(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

const BUFFER_SIZE: usize = 4096;

pub struct TcpConnection {
    stream: Option<TcpStream>,
    listener: Option<TcpListener>,
    address: String,
    port: u16,
}

impl TcpConnection {
    pub fn new(address: String, port: u16) -> Self {
        Self { stream: None, listener: None, address, port }
    }
}

impl Connection for TcpConnection {
    async fn open(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let ip = if self.address.contains(":") {
            format!("[{}]:{}", self.address, self.port)
        } else {
            format!("{}:{}", self.address, self.port)
        };

        self.stream = Some(TcpStream::connect(ip.as_str()).await?);

        Ok(())
    }

    async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.listener = Some(TcpListener::bind(format!("{}:{}", self.address, self.port)).await?);

        Ok(())
    }

    async fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(stream) = &mut self.stream { 
            stream.shutdown().await?;
            self.stream = None;
        }
        if let Some(_) = &mut self.listener {
            self.listener = None;
        }
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(data).await?;
        } else {
            return Err("No active connection to send data".into());
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Some(stream) = &mut self.stream {
            let mut buffer = vec![0; BUFFER_SIZE];
            let n = stream.read(&mut buffer).await?;
            buffer.truncate(n);
            Ok(buffer)
        } else if let Some(listener) = &mut self.listener {
            let (mut stream, _) = listener.accept().await?;
            let mut buffer = vec![0; BUFFER_SIZE];
            let n = stream.read(&mut buffer).await?;
            buffer.truncate(n);
            Ok(buffer)
        } else {
            Err("No active connection to receive data".into())
        }
    }

    fn get_ip(&self) -> String {
        self.address.clone()
    }

    fn get_port(&self) -> u16 {
        self.port
    }

    fn set_ip(&mut self, ip: String) {
        self.address = ip;
    }

    fn set_port(&mut self, port: u16) {
        self.port = port;
    }
}