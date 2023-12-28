use arti_client::TorClient;
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{upgrade::Upgraded, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tor_rtcompat::PreferredRuntime;

#[tracing::instrument(skip(tor_client))]
pub async fn proxy(
    req: Request<hyper::body::Incoming>,
    tor_client: &'static TorClient<PreferredRuntime>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
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

        Ok(Response::new(empty()))
    } else {
        tracing::warn!("CONNECT host is not socket addr: {:?}", req.uri());
        let mut resp = Response::new(full("CONNECT must be to a socket address"));
        *resp.status_mut() = StatusCode::BAD_REQUEST;

        Ok(resp)
    }
}

async fn tunnel(
    upgraded: Upgraded,
    addr: String,
    tor_client: &'static TorClient<PreferredRuntime>,
) -> std::io::Result<()> {
    let tor_client = tor_client.isolated_client();

    let mut server = tor_client.connect(addr).await.unwrap();
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    tracing::debug!(
        "client wrote {} bytes and received {} bytes",
        from_client,
        from_server
    );

    Ok(())
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
