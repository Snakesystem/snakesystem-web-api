use actix::{Actor, StreamHandler, AsyncContext, Handler, Message, Addr};
use actix_web_actors::ws::{self, WebsocketContext, Message as WsMessage};
use serde_json::Value;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref CLIENTS: Mutex<Vec<Addr<WsSession>>> = Mutex::new(vec![]);
}

pub fn register(addr: Addr<WsSession>) {
    let mut clients = CLIENTS.lock().unwrap();
    clients.push(addr);
}

pub fn broadcast(message: WsPushEvent) {
    let clients = CLIENTS.lock().unwrap();
    for client in clients.iter() {
        let _ = client.do_send(message.clone());
    }
}

pub struct WsSession;

impl WsSession {
    pub fn new() -> Self {
        WsSession
    }
}

impl Actor for WsSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        register(ctx.address());
    }
}

impl StreamHandler<Result<WsMessage, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<WsMessage, ws::ProtocolError>, _ctx: &mut WebsocketContext<Self>) {
        if let Ok(WsMessage::Text(text)) = msg {
            println!("Client sent: {}", text);
        }
    }
}

impl Handler<WsPushEvent> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsPushEvent, ctx: &mut WebsocketContext<Self>) {
        let json = serde_json::json!({
            "event": msg.event,
            "data": msg.data
        });
        ctx.text(json.to_string());
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WsPushEvent {
    pub event: String,
    pub data: Value,
}

// Fungsi utama yang bisa dipanggil dari luar
pub fn send_ws_event(event: &str, data: impl serde::Serialize) {
    if let Ok(json) = serde_json::to_value(data) {
        broadcast(WsPushEvent {
            event: event.to_string(),
            data: json,
        });
    }
}
