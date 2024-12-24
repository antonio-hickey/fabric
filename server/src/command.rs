use crate::{Error, ThreadSafeFabric};

/// The different types of supported commands
pub enum Command {
    /// Get a data structure
    Get,
    /// Set a data structure
    Set,
}
impl Command {
    /// Initialize a command from client input
    pub fn from(input: &str) -> Result<Command, Error> {
        let trimmed = input.trim();
        if trimmed.starts_with("GET") {
            Ok(Command::Get)
        } else if trimmed.starts_with("SET") {
            Ok(Command::Set)
        } else {
            let parts: Vec<&str> = trimmed.split(" ").collect();
            let cmd = parts[0].to_string();

            Err(Error::UnsupportedCommand(cmd))
        }
    }

    /// Handle the functionality behind a command
    pub async fn handle(&self, line: &str, fabric: &ThreadSafeFabric) -> Result<Vec<u8>, Error> {
        match self {
            Command::Get => {
                let key = line.strip_prefix("GET ").unwrap_or("");
                let value = {
                    let data = fabric.read().await;
                    data.structures
                        .get(key)
                        .cloned()
                        .ok_or_else(|| Error::KeyNotFound(key.to_string()))
                }?;

                let bytes = value.to_string().into_bytes();

                Ok(bytes)
            }
            Command::Set => {
                let parts: Vec<&str> = line
                    .strip_prefix("SET ")
                    .unwrap_or("")
                    .splitn(2, ' ')
                    .collect();

                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    {
                        let mut data = fabric.write().await;
                        let value = serde_json::from_str(&value)?;

                        data.structures.insert(key, value);
                    }

                    Ok("OK\n".as_bytes().to_vec())
                } else {
                    Ok("Invalid SET Command\n".as_bytes().to_vec())
                }
            }
        }
    }
}