use arti_client::TorClient;
use axum::{
    body,
    response::{IntoResponse, Response},
};
use hyper::{upgrade::Upgraded, Body, Request, StatusCode};
use tor_rtcompat::PreferredRuntime;

#[tracing::instrument(skip(tor_client))]
pub async fn proxy(
    req: Request<Body>,
    tor_client: &'static TorClient<PreferredRuntime>,
) -> Result<Response, hyper::Error> {
    if let Some(host_addr) = req.uri().authority().map(ToString::to_string) {
        tracing::debug!("{}", host_addr);
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, host_addr, tor_client).await {
                        tracing::error!("server io error: {}", e);
                    };
                }
                Err(e) => tracing::error!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(body::boxed(body::Empty::new())))
    } else {
        tracing::error!("CONNECT host is not socket addr: {:?}", req.uri());
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
    tor_client: &'static TorClient<PreferredRuntime>,
) -> std::io::Result<()> {
    let tor_client = tor_client.isolated_client();

    let mut server = tor_client.connect(addr).await.unwrap();

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    tracing::debug!(
        "client wrote {} bytes and received {} bytes",
        from_client,
        from_server
    );

    Ok(())
}
