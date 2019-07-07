use crate::api;
use crate::server::State;
use actix_web::dev::HttpResponseBuilder;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse};

#[post("/api")]
pub fn api_handler(shared_state: Data<State>, request: Json<api::Request>) -> HttpResponse {
    to_http_response(api::handle_request(
        &shared_state.player,
        &shared_state.library,
        request.0,
    ))
}

fn to_http_response(json_result: api::JsonResult) -> HttpResponse {
    use http::status::StatusCode;
    let (status, body) = match json_result {
        Ok(s) => (StatusCode::OK, s),
        Err(s) => (StatusCode::INTERNAL_SERVER_ERROR, s),
    };
    HttpResponseBuilder::new(status)
        .content_type("application/json")
        .body(body)
}
