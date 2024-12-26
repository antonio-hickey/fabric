use crate::{Error, ThreadSafeFabric};

/// The different types of supported commands
pub enum Command {
    /// Get an entry in cache
    Get,
    /// Set an entry in cache
    Set,
    /// Remove an entry from cache
    Remove,
}
impl Command {
    /// Initialize a command from client input
    pub fn from(input: &str) -> Result<Command, Error> {
        let trimmed = input.trim();
        if trimmed.starts_with("GET") {
            Ok(Command::Get)
        } else if trimmed.starts_with("SET") {
            Ok(Command::Set)
        } else if trimmed.starts_with("REMOVE") {
            Ok(Command::Remove)
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
                let keys = key.split('.').collect();
                let value = fabric.read().await.get(keys)?;
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
                    let keys = key.split('.').collect();
                    let value = parts[1].to_string();

                    fabric.write().await.set(keys, &value)?;

                    Ok("OK\n".as_bytes().to_vec())
                } else {
                    Ok("Invalid SET Command\n".as_bytes().to_vec())
                }
            }
            Command::Remove => {
                let key = line.strip_prefix("REMOVE ").unwrap_or("");
                let keys = key.split('.').collect();

                fabric.write().await.remove(keys)?;

                Ok("OK\n".as_bytes().to_vec())
            }
        }
    }
}
