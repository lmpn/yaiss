use std::{net::SocketAddr, time::Duration};

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
pub struct ServerHandler {
    handle: Handle,
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
