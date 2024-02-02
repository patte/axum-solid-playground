use axum::{
    body::Body,
    extract::{Request, State},
    http::uri::Uri,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

pub fn get_client() -> Client {
    hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
        .build(HttpConnector::new())
}

pub async fn proxy_handler(
    State(client): State<Client>,
    mut req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://localhost:3001{}", path_query);

    //info!("Proxying request to {}", uri);
    *req.uri_mut() = Uri::try_from(uri).unwrap();

    Ok(client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_response())
}
