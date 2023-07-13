use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    http::{
        header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, ORIGIN},
        Method,
    },
    routing::get,
    Router,
};
use axum_server::Handle;
use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::configuration::Configuration;

pub struct ServerHandler {
    handle: Option<Handle>,
    address: SocketAddr,
    router: Router<()>,
}

impl ServerHandler {
    pub fn new() -> Self {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET])
            .allow_headers([AUTHORIZATION, ORIGIN, ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN]);
        let router = Router::<()>::new().route("/", get(get_home)).layer(cors);
        let address = ([0, 0, 0, 0], 3000);
        let sock_address = SocketAddr::from(address);
        Self {
            handle: Some(Handle::new()),
            address: sock_address,
            router,
        }
    }

    pub fn serve(&mut self) {
        let handle = match &self.handle {
            Some(handle) => handle.clone(),
            None => {
                let handle = Handle::new();
                self.handle = Some(handle.clone());
                handle
            }
        };
        let server = axum_server::bind(self.address)
            .handle(handle)
            .serve(self.router.clone().into_make_service());
        tokio::spawn(async {
            event!(Level::INFO, "Starting server");
            server.await.unwrap();
        });
    }

    pub async fn reload(&mut self, configuration: Configuration) {
        let sock_address = SocketAddr::from(configuration.address());
        if self.address == sock_address {
            return;
        }
        self.stop().await;

        self.address = sock_address;

        self.serve()
    }

    pub async fn stop(&mut self) {
        if let None = self.handle {
            return;
        }
        let handle = self.handle.take().unwrap();
        handle.graceful_shutdown(Some(Duration::from_secs(3)));
        let mut conn_count = handle.connection_count();
        while conn_count > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            conn_count = handle.connection_count();
        }
        event!(Level::INFO, "Stopping server");
    }
}

async fn get_home() -> &'static str {
    "Hello world"
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn start_and_stop() {
        let mut sh = ServerHandler::new();
        sh.serve();
        sleep(Duration::from_millis(500)).await;
        let response = reqwest::get("http://0.0.0.0:3000/")
            .await
            .expect("failed to perfrom GET /")
            .text()
            .await
            .expect("failed to read payload");
        assert_eq!(response, "Hello world");
        sh.stop().await;

        let response = reqwest::get("http://0.0.0.0:3000/")
            .await
            .expect_err("expected error");
        assert!(response.is_request());
    }
}
