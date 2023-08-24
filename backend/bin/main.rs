use std::error::Error;

use tokio::signal::unix::{signal, SignalKind};
use tracing::{event, Level};
use yaiss_backend::{configuration::Configuration, server::Server, state::State};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    let mut configuration = Configuration::new();
    let state = State::new(&configuration);
    let mut server_handler = Server::new(state, &configuration);
    server_handler.serve();

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                event!(Level::INFO, "SIGTERM Shuting down");
                break;
            }
            _ = sigint.recv() => {
                event!(Level::INFO,"SIGINT Shuting down");
                break;
            },
            Some(()) = configuration.has_change() => {
                event!(Level::INFO,"Configuration changed");
                let configuration = Configuration::new();
                let state = State::new(&configuration);
                server_handler.reload(state, configuration).await
            }
        };
    }

    server_handler.stop().await;
    Ok(())
}
