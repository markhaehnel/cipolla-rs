use lazy_static::lazy_static;
use std::net::SocketAddr;

use anyhow::Result;
use arti_client::{BootstrapBehavior, TorClient};
use axum::{
    body::{self, Body},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use hyper::upgrade::Upgraded;
use tower::{make::Shared, ServiceExt};

lazy_static! {
    static ref TOR_CLIENT: TorClient<tor_rtcompat::PreferredRuntime> = TorClient::builder()
        .bootstrap_behavior(BootstrapBehavior::OnDemand)
        .create_unbootstrapped()
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut handles = Vec::new();

    let num_ports = 100;
    let port_from = 8000;
    let port_to = port_from + num_ports + 1;

    let router_svc = Router::new().route("/", get(|| async { "Hello, World!" }));

    let service = tower::service_fn(move |req: Request<Body>| {
        let router_svc = router_svc.clone();
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
            let addr = SocketAddr::from(([127, 0, 0, 1], port));

            axum::Server::bind(&addr)
                .http1_preserve_header_case(true)
                .http1_title_case_headers(true)
                .serve(Shared::new(service_clone))
                .await
        });
        handles.push(job);
    }

    println!("Listening on ports from {} to {}", port_from, port_to);

    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}

async fn proxy(
    req: Request<Body>,
    tor_client: &'static TorClient<tor_rtcompat::PreferredRuntime>,
) -> Result<Response, hyper::Error> {
    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        println!("{}", host_addr);
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, host_addr, tor_client).await {
                        eprintln!("server io error: {}", e);
                    };
                }
                Err(e) => eprintln!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(body::boxed(body::Empty::new())))
    } else {
        eprintln!("CONNECT host is not socket addr: {:?}", req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        )
            .into_response())
    }
}

async fn tunnel(
    mut upgraded: Upgraded,
    addr: String,
    tor_client: &'static TorClient<tor_rtcompat::PreferredRuntime>,
) -> std::io::Result<()> {
    let tor_client = tor_client.isolated_client();

    let mut server = tor_client.connect(addr).await.unwrap();

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}
