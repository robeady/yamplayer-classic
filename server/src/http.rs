use crate::api;
use std::sync::Arc;
use warp::http::header::CONTENT_TYPE;
use warp::http::status::StatusCode;
use warp::http::Response;
use warp::Reply;

pub fn api_handler(app: Arc<api::App>, request: api::Request) -> impl Reply {
    to_http_response(app.handle_request(&request))
}

fn to_http_response(result: api::Response) -> impl Reply {
    let (status, body) = match result {
        Ok(p) => (StatusCode::OK, p.json),
        Err(p) => (StatusCode::INTERNAL_SERVER_ERROR, p.json),
    };
    Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .status(status)
        .body(body)
}
