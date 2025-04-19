use crate::Error;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

/// Client for interacting with your fabric server
pub struct FabricClient {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}
impl FabricClient {
    /// Open a connection to your fabric server.
    pub async fn connect(addr: &str) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr).await?;
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);
        let writer = BufWriter::new(write_half);
        Ok(FabricClient { reader, writer })
    }

    /// Perform the SET command on a provided key to
    /// either insert, or update the value of the key.
    ///
    /// NOTE: That any data structure `T` for the value
    /// must implement the `serde::Serialize` trait.
    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Error> {
        let serialized_data = serde_json::to_string(value).map_err(Error::BadDataStructure)?;

        let command = format!("SET {} {}\n", key, serialized_data);
        self.writer.write_all(command.as_bytes()).await?;
        self.writer.flush().await?;

        let mut resp = String::new();
        let bytes = self.reader.read_line(&mut resp).await?;

        if bytes == 0 {
            return Err(Error::Unknown("Disconnected".into()));
        }

        if resp.contains("OK") {
            Ok(())
        } else {
            Err(Error::Unknown(resp))
        }
    }

    /// Perform the GET command on a provided key to
    /// grab the current value of the key.
    ///
    /// NOTE: You must specify your return type and it
    /// needs to implement the `serde::Deserialize` trait.
    pub async fn get<S: Into<String>, T>(&mut self, key: S) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let command = format!("GET {}\n", key.into());
        self.writer.write_all(command.as_bytes()).await?;
        self.writer.flush().await?;

        let mut resp = String::new();
        self.reader.read_line(&mut resp).await?;

        let value: T = serde_json::from_str(&resp)?;
        Ok(value)
    }

    /// Perform the REMOVE command on a provided key to
    /// remove the key/value pair from cache.
    pub async fn remove(&mut self, key: &str) -> Result<(), Error> {
        let command = format!("REMOVE {}\n", key);
        self.writer.write_all(command.as_bytes()).await?;
        self.writer.flush().await?;

        let mut resp = String::new();
        self.reader.read_line(&mut resp).await?;

        if resp.contains("OK") {
            Ok(())
        } else {
            Err(Error::Unknown(resp))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn mock_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buffer = [0; 512];
                let n = socket.read(&mut buffer).await.unwrap();
                let command = String::from_utf8_lossy(&buffer[..n]);
                let response = if command.starts_with("SET") {
                    "OK\n"
                } else if command.starts_with("GET") {
                    r#""value""#
                } else if command.starts_with("REMOVE") {
                    "OK\n"
                } else {
                    "ERROR\n"
                };
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
        addr.to_string()
    }

    #[tokio::test]
    async fn test_set_command() {
        let addr = mock_server().await;
        let mut client = FabricClient::connect(&addr).await.unwrap();

        let key = "test_key";
        let value = json!({"data": "value"});

        let result = client.set(key, &value).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_command() {
        let addr = mock_server().await;
        let mut client = FabricClient::connect(&addr).await.unwrap();

        let key = "test_key";
        let result: Result<String, Error> = client.get(key).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, json!("value"));
    }

    #[tokio::test]
    async fn test_remove_command() {
        let addr = mock_server().await;
        let mut client = FabricClient::connect(&addr).await.unwrap();

        let key = "test_key";
        let result = client.remove(key).await;

        assert!(result.is_ok());
    }
}
