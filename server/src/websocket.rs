use crate::api;
use crate::api::{App, EventDestination, Payload};
use crate::library::Library;
use futures::future;
use futures::sync::mpsc::{unbounded, UnboundedSender};
use serde_derive::Deserialize;
use std::sync::Arc;
use warp::ws::{Message, WebSocket};
use warp::{Future, Sink, Stream};

pub fn ws_connection<L: Library>(
    app: Arc<App<L>>,
    websocket: WebSocket,
) -> impl Future<Item = (), Error = ()> {
    log::info!("establishing WS connection");

    let (socket_tx, socket_rx) = websocket.split();
    // establish a queue for outbound messages to the websocket
    let (outbound_tx, outbound_rx) = unbounded::<Message>();
    let outbound_tx = Arc::new(outbound_tx);

    // first, send stuff from `outbound_rx` to the websocket
    tokio::spawn(
        outbound_rx
            .forward(socket_tx.sink_map_err(move |e| panic!("websocket send error: {}", e)))
            .map(|_| ()),
    );

    // second, hook up this socket to the event sink
    let key = app
        .event_sink
        .add_destination(Box::new(outbound_tx.clone()));

    // third, handle requests from the websocket
    let app2 = app.clone();
    // return this and let warp spawn it
    socket_rx
        .for_each(move |message| {
            // ignore non-text messages
            if let Ok(message) = message.to_str() {
                let outbound_tx = outbound_tx.clone();
                tokio::spawn(
                    handle_message(app.clone(), message).map(move |response_message| {
                        // do nothing if the channel was closed
                        let _ = outbound_tx.unbounded_send(response_message);
                    }),
                );
            }
            Ok(())
        })
        .then(move |r| {
            log::info!("WS connection closed");
            app2.event_sink.remove_destination(key);
            r
        })
        .map_err(|e| panic!("websocket receive error: {}", e))
}

#[derive(Debug, Deserialize)]
struct WebSocketRequest(String, api::Request);

fn handle_message<L: Library>(
    app: Arc<App<L>>,
    message_text: &str,
) -> impl Future<Item = Message, Error = ()> {
    let WebSocketRequest(id, request) =
        serde_json::from_str(message_text).expect("invalid json in websocket message");
    future::poll_fn(move || tokio_threadpool::blocking(|| app.handle_request(&request)))
        .map(|response| websocket_response(id, response))
        .map_err(|e| panic!("outside tokio: {}", e))
}

fn websocket_response(id: String, result: api::Response) -> Message {
    Message::text(match result {
        Ok(p) => format!("[\"{}\",{}]", id, p.json),
        Err(p) => format!("[\"{}\",null,{}]", id, p.json),
    })
}

fn websocket_notification(payload: &Payload) -> Message {
    Message::text(format!("[\"\",{}]", payload.json))
}

impl EventDestination for Arc<UnboundedSender<Message>> {
    fn send_event(&self, payload: &Payload) {
        // ignore error, don't care if channel was disconnected, the handler will tidy this up
        let _ = self.unbounded_send(websocket_notification(payload));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::Payload;
    use crate::websocket::websocket_response;
    use warp::ws::Message;

    #[test]
    fn websocket_response_returns_array() {
        assert_eq!(
            websocket_response(
                "hello".to_string(),
                Ok(Payload {
                    json: "{}".to_string()
                })
            ),
            Message::text("[\"hello\",{}]")
        );
        assert_eq!(
            websocket_response(
                "hello".to_string(),
                Err(Payload {
                    json: "{}".to_string()
                })
            ),
            Message::text("[\"hello\",null,{}]")
        )
    }
}
