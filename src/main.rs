use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8731").await?;

    loop {
        let (socket, addr) = tcp_listener.accept().await?;

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut lines = BufReader::new(reader).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                println!("Received from {:?}: {}", addr, line);

                if let Err(e) = writer.write_all(format!("{}\n", line).as_bytes()).await {
                    eprintln!("Failed to write to {}: {}", addr, e);
                    break;
                }
            }

            println!("Connection with {:?} closed.", addr);
        });
    }
}
