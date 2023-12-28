#![warn(clippy::pedantic)]
#![feature(lazy_cell)]

use std::{net::SocketAddr, sync::LazyLock};

use anyhow::Result;
use arti_client::TorClient;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tor_rtcompat::PreferredRuntime;
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
                .unwrap_or_else(|_| "cipolla=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut handles = Vec::new();

    let num_ports = ARGS.count;
    let port_from = ARGS.port;
    let port_to = port_from + num_ports;

    for port in port_from..port_to {
        let job = tokio::spawn(async move {
            let addr: SocketAddr = format!("[::]:{port}").parse().expect("Invalid port");

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);

                let _ = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(io, service_fn(|req| proxy(req, &TOR_CLIENT)))
                    .with_upgrades()
                    .await;
            }
        });

        handles.push(job);
    }

    tracing::info!("listening on ports {} to {}", port_from, port_to - 1);

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
