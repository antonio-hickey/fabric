mod command;
mod error;
mod fabric;

use self::{command::Command, error::Error, fabric::Fabric};
use anyhow::Result;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

pub type ThreadSafeFabric = Arc<RwLock<Fabric>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Start listening for TCP connections at localhost on port 8731
    // TODO / NOTE: This should be configurable, both the ip address and the port.
    let tcp_listener = TcpListener::bind("127.0.0.1:8731").await?;

    // Initialize a thread safe instance of `Fabric`
    let fabric: ThreadSafeFabric = Arc::new(RwLock::new(Fabric::new()));

    loop {
        // Accept incoming TCP connections into a socket (TCP Stream)
        let (socket, _) = tcp_listener.accept().await?;

        // The shared data structure store between clients
        let fabric = fabric.clone();

        // Start the server and handle the client streams
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, fabric).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}

/// Handle a client's TCP stream.
async fn handle_client(socket: TcpStream, fabric: ThreadSafeFabric) -> Result<(), Error> {
    // The IO for the TCP stream between client and server
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut client_input = String::new();

    loop {
        // Read the client input from the tcp stream
        client_input.clear();
        let bytes_read = reader.read_line(&mut client_input).await?;
        if bytes_read == 0 {
            // Client disconnected
            break;
        }
        let client_input = client_input.trim();

        // Parse the client input into a `Command` and handle the
        // functionality behind the command returning the output
        // to then send back to the client.
        let cmd = Command::from(client_input)?;
        let output = cmd.handle(client_input, &fabric).await?;
        writer.write_all(&output).await?;
    }

    Ok(())
}
