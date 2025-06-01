mod handle_client;

use tokio::net::TcpListener;
use tokio::sync::broadcast;
use std::error::Error;
use handle_client::handle_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let port = 8000;
    let server_address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(server_address).await?;
    
    log::info!("Server running on {}", listener.local_addr()?);
    let (tx, _) = broadcast::channel(100);
    
    loop {
        let (socket, addr) = listener.accept().await?;
        log::info!("{} connected to server", addr);

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, addr, tx, rx).await {
                log::error!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

