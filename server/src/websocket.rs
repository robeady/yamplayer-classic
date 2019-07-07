use crate::api;
use crate::api::JsonResult;
use crate::server::State;
use actix::prelude::ActorContext;
use actix::{Actor, StreamHandler};
use actix_web::web::Data;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde_derive::Deserialize;

/// do websocket handshake and start `MyWebSocket` actor

#[get("/ws")]
pub fn index(
    shared_state: Data<State>,
    r: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let state = shared_state.clone();
    ws::start(WebSocket(ApiHandler { state }), &r, stream)
}

trait Handler {
    fn handle(&self, message: &str) -> Option<String>;
}

struct WebSocket<T>(T);

impl<T: 'static> Actor for WebSocket<T> {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for `ws::Message`
impl<T: 'static + Handler> StreamHandler<ws::Message, ws::ProtocolError> for WebSocket<T> {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        use ws::Message::*;
        match msg {
            Ping(msg) => ctx.pong(&msg),
            Pong(_) => (),
            Text(ref text) => {
                if let Some(reply) = self.0.handle(text) {
                    ctx.text(reply)
                }
            }
            Binary(_) => panic!("binary messages not supported"),
            Close(_) => ctx.stop(),
            Nop => (),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WebSocketRequest {
    id: Option<String>,
    payload: api::Request,
}

struct ApiHandler {
    state: Data<State>,
}

impl Handler for ApiHandler {
    fn handle(&self, message: &str) -> Option<String> {
        let request: WebSocketRequest = serde_json::from_str(message).unwrap();
        let result = api::handle_request(&self.state.player, &self.state.library, request.payload);
        request.id.map(|id| websocket_response(id, result))
    }
}

fn websocket_response(id: String, result: JsonResult) -> String {
    match result {
        Ok(s) => format!("[\"{}\",{}]", id, s),
        Err(s) => format!("[\"{}\",null,{}]", id, s),
    }
}

#[cfg(test)]
mod tests {
    use crate::websocket::websocket_response;

    #[test]
    fn websocket_response_returns_array() {
        assert_eq!(
            websocket_response("hello".to_string(), Ok("{}".to_string())),
            "[\"hello\",{}]"
        );
        assert_eq!(
            websocket_response("hello".to_string(), Err("{}".to_string())),
            "[\"hello\",null,{}]"
        )
    }
}
