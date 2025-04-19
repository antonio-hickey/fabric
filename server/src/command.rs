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
                match fabric.read().await.get(keys) {
                    Ok(value) => Ok(format!("{}\n", value).into_bytes()),
                    Err(_) => Ok(format!("{}\n", Error::KeyNotFound(key.into())).into_bytes()),
                }
            }
            Command::Set => {
                let line = line.trim_end();
                let cmd_str = line.strip_prefix("SET ").unwrap_or("");
                let parts: Vec<&str> = cmd_str.splitn(2, ' ').collect();

                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let keys = key.split('.').collect();
                    let value = parts[1].to_string();

                    match fabric.write().await.set(keys, &value) {
                        Ok(_) => Ok(b"OK\n".to_vec()),
                        Err(e) => Ok(format!("SET ERROR For Key: {key}: {e:?}").into_bytes()),
                    }
                } else {
                    Ok(b"Invalid SET Command\n".to_vec())
                }
            }
            Command::Remove => {
                let key = line.strip_prefix("REMOVE ").unwrap_or("");
                let keys = key.split('.').collect();

                match fabric.write().await.remove(keys) {
                    Ok(_) => Ok(b"OK\n".to_vec()),
                    Err(e) => Ok(format!("REMOVE Error For Key: {key}: {e:?}").into_bytes()),
                }
            }
        }
    }
}
