use std::error::Error as StdErrorTrait;

#[derive(Debug)]
/// The different types of error's in Fabric.
pub enum Error {
    KeyNotFound(String),
    IO(std::io::Error),
    BadDataStructure(serde_json::Error),
    UnsupportedCommand(String),
    InvalidKeyPath(String),
}
impl StdErrorTrait for Error {}
/// Implement display trait for `Error`
impl std::fmt::Display for Error {
    /// The error message display format
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::KeyNotFound(key) => write!(f, "Key: \"{}\" Not Found.", key),
            Error::IO(e) => write!(f, "IO Error:\n {}", e),
            Error::BadDataStructure(e) => write!(f, "Bad Data Structure: Error:\n {}", e),
            Error::UnsupportedCommand(cmd) => write!(f, "\"{}\" Is Not A Supported Command.", cmd),
            Error::InvalidKeyPath(key_path) => {
                write!(f, "\"{}\" Is Not A Valid Key Path.", key_path)
            }
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::BadDataStructure(err)
    }
}
