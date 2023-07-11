use std::{error::Error, net::SocketAddr, time::Duration};

use axum::{
    http::{
        header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, ORIGIN},
        Method,
    },
    routing::get,
    Router,
};
use axum_server::Handle;
use tokio::signal::unix::{signal, SignalKind};
use tower_http::cors::{Any, CorsLayer};
use tracing::{event, span, Level};
use tracing_subscriber::filter::LevelFilter;
struct Server {
    handle: Handle,
    address: SocketAddr,
    router: Router<()>,
}

impl Server {
    fn new() -> Self {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET])
            .allow_headers([AUTHORIZATION, ORIGIN, ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN]);
        let router = Router::<()>::new().route("/", get(get_home)).layer(cors);
        let address = SocketAddr::from(([0, 0, 0, 0], 3000));
        Self {
            handle: Handle::new(),
            address,
            router,
        }
    }

    pub fn serve(&mut self) {
        let server = axum_server::bind(self.address)
            .handle(self.handle.clone())
            .serve(self.router.clone().into_make_service());
        tokio::spawn(async {
            server.await.unwrap();
        });
    }

    pub async fn stop(&self) {
        self.handle.graceful_shutdown(Some(Duration::from_secs(15)));
        let mut conn_count = self.handle.connection_count();
        while conn_count > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            conn_count = self.handle.connection_count();
        }
    }
}

async fn get_home() -> &'static str {
    "Hello world"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    let mut server = Server::new();
    server.serve();

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = sigterm.recv() => {event!(Level::INFO, "SIGTERM Shuting down");}
        _ = sigint.recv() => {event!(Level::INFO,"SIGINT Shuting down");},
    };

    server.stop().await;
    Ok(())
}
