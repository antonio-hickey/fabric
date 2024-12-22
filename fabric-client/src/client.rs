use crate::Error;
use serde::Serialize;
use serde_json::Value;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Client for interacting with your fabric server
pub struct FabricClient {
    stream: TcpStream,
}
impl FabricClient {
    /// Open a connection to your fabric server.
    pub async fn connect(addr: &str) -> tokio::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(FabricClient { stream })
    }

    /// Perform the SET command on a provided key to
    /// either insert, or update the value of the key.
    ///
    /// NOTE: That any data structure `T` for the value
    /// must implement the `serde::Serialize` trait.
    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Error> {
        let serialized_data = serde_json::to_string(value).map_err(Error::BadDataStructure)?;

        let command = format!("SET {} {}\n", key, serialized_data);
        self.stream.write_all(command.as_bytes()).await?;
        self.stream.flush().await?;

        let mut buffer = vec![0; 512];
        let n = self.stream.read(&mut buffer).await?;

        let resp = String::from_utf8_lossy(&buffer[..n]);
        if resp.contains("OK") {
            Ok(())
        } else {
            Err(Error::Unknown(resp.to_string()))
        }
    }

    /// Perform the GET command on a provided key to
    /// grab the current value of the key.
    pub async fn get<S: Into<String>>(&mut self, key: &str) -> Result<Value, Error> {
        let command = format!("GET {}\n", key);
        self.stream.write_all(command.as_bytes()).await?;
        self.stream.flush().await?;

        let mut buffer = vec![0; 512];
        let n = self.stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]).to_string();

        let value = serde_json::from_str(&response)?;
        Ok(value)
    }
}
