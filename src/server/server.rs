use std::net::SocketAddr;
use std::time::Duration;

use axum::routing::get;
use axum::{
    http::{
        header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, ORIGIN},
        Method,
    },
    Router,
};
use axum_server::Handle;
use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

use crate::configuration::Configuration;
use crate::images;
use crate::state::State;

pub struct Server {
    handle: Option<Handle>,
    address: SocketAddr,
    router: Router<()>,
}

impl Server {
    pub fn new(state: State, configuration: &Configuration) -> Self {
        let router = Self::create_router(state);
        let sock_address = SocketAddr::from(configuration.address());
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

    pub async fn reload(&mut self, state: State, configuration: Configuration) {
        let sock_address = SocketAddr::from(configuration.address());
        if self.address == sock_address {
            return;
        }
        self.stop().await;

        self.address = sock_address;
        self.router = Self::create_router(state);

        self.serve()
    }

    pub async fn stop(&mut self) {
        if self.handle.is_none() {
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

    fn create_router(state: State) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET])
            .allow_headers([AUTHORIZATION, ORIGIN, ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN]);
        Router::new()
            .route("/", get(hello_world))
            .merge(images::web::router())
            .with_state(state)
            .layer(cors)
    }
}

async fn hello_world() -> &'static str {
    "Hello world!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn start_and_stop() {
        let configuration = Configuration::new();
        let state = State::new(&configuration);
        let mut sh = Server::new(state, &configuration);
        sh.serve();
        sleep(Duration::from_millis(500)).await;
        let response = reqwest::get("http://0.0.0.0:3000/")
            .await
            .expect("failed to perfrom GET /")
            .text()
            .await
            .expect("failed to read payload");
        assert_eq!(response, "Hello world!");
        sh.stop().await;

        let response = reqwest::get("http://0.0.0.0:3000/")
            .await
            .expect_err("expected error");
        assert!(response.is_request());
    }
}
