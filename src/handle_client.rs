use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use std::error::Error;


pub async fn handle_client(
    socket: TcpStream,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) -> Result<(), Box<dyn Error>> {
    let (mut reader, mut writer) = socket.into_split();

    let read_handle = tokio::spawn(async move {
        let mut buf = [0; 1024];

        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    log::info!("{} disconnected", addr);

                    break
                },
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                    log::debug!("{}: {}", addr, msg);

                    if let Err(e) = tx.send(format!("{}: {}", addr, msg)) {
                        log::error!("Error broadcasting message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Error reading from {}: {}", addr, e);
                    break;
                }
            }
        }
    });

    let write_handle = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        log::error!("Error writing to {}: {}", addr, e);
                        break;
                    }
                    if let Err(e) = writer.write_all(b"").await {
                        log::error!("Error writing newline to {}: {}", addr, e);
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Error receiving broadcast for {}: {}", addr, e);
                    break;
                }
            }
        }
    });

    tokio::try_join!(read_handle, write_handle)?;

    log::info!("Connection closed: {}", addr);
    Ok(())
}