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
    pub async fn get<S: Into<String>>(&mut self, key: S) -> Result<Value, Error> {
        let command = format!("GET {}\n", key.into());
        self.stream.write_all(command.as_bytes()).await?;
        self.stream.flush().await?;

        let mut buffer = vec![0; 512];
        let n = self.stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]).to_string();

        let value = serde_json::from_str(&response)?;
        Ok(value)
    }

    /// Perform the REMOVE command on a provided key to
    /// remove the key/value pair from cache.
    pub async fn remove(&mut self, key: &str) -> Result<(), Error> {
        let command = format!("REMOVE {}\n", key);
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
        let result = client.get(key).await;

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
