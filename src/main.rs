#![warn(clippy::pedantic)]
#![feature(lazy_cell)]

use std::{net::SocketAddr, sync::LazyLock};

use anyhow::Result;
use arti_client::TorClient;
use axum::{
    body::Body,
    http::{Method, Request},
    Router,
};
use tor_rtcompat::PreferredRuntime;
use tower::{make::Shared, ServiceExt};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cipolla::{cli, proxy::proxy, tor::build_tor_client};

static ARGS: LazyLock<cli::Cli> = LazyLock::new(cli::parse);

static TOR_CLIENT: LazyLock<TorClient<PreferredRuntime>> =
    LazyLock::new(|| build_tor_client(ARGS.exit_country.as_deref()));

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cipolla=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut handles = Vec::new();

    let num_ports = ARGS.count;
    let port_from = ARGS.port;
    let port_to = port_from + num_ports;

    let router = Router::new().layer(TraceLayer::new_for_http());

    let service = tower::service_fn(move |req: Request<Body>| {
        let router_svc = router.clone();
        async move {
            if req.method() == Method::CONNECT {
                proxy(req, &TOR_CLIENT).await
            } else {
                router_svc.oneshot(req).await.map_err(|err| match err {})
            }
        }
    });

    for port in port_from..port_to {
        let service_clone = service.clone();
        let job = tokio::spawn(async move {
            let addr: SocketAddr = format!("[::]:{port}").parse().expect("Invalid port");

            axum::Server::bind(&addr)
                .http1_preserve_header_case(true)
                .http1_title_case_headers(true)
                .serve(Shared::new(service_clone))
                .await
        });

        handles.push(job);
    }

    tracing::info!("listening on ports {} to {}", port_from, port_to - 1);

    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
