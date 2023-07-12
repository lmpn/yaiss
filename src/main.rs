use std::error::Error;

use tokio::signal::unix::{signal, SignalKind};
use tracing::{event, Level};

use crate::server::server_handler::ServerHandler;
mod images;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    let mut server_handler = ServerHandler::new();
    server_handler.serve();

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = sigterm.recv() => {event!(Level::INFO, "SIGTERM Shuting down");}
        _ = sigint.recv() => {event!(Level::INFO,"SIGINT Shuting down");},
    };

    server_handler.stop().await;
    Ok(())
}
