use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use actix::prelude::Addr;
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{web, HttpResponse};
use actix_web_actors::ws;
use mongodb::bson::oid::ObjectId;
use std::time::{Duration, Instant};

use crate::{auth, error::ServerResponse, routes::payloads::WebsocketMessage, WebsocketState};
// TODO: Add a userId to this struct for a bit of state
// using this userid it will be possible to subscribe to the correct topic
// again possibly using the new datastructure that is storing the topic names
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct ProjectUpdateWs {
    hb: Instant,
    id: Option<ObjectId>,
    map: Arc<Mutex<HashMap<ObjectId, Addr<ProjectUpdateWs>>>>,
}

impl Actor for ProjectUpdateWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx)
    }
}

impl ProjectUpdateWs {
    pub fn new(map: Arc<Mutex<HashMap<ObjectId, Addr<ProjectUpdateWs>>>>) -> ProjectUpdateWs {
        ProjectUpdateWs {
            id: None,
            hb: Instant::now(),
            map: map,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::info!("Disconnecting failed heartbeat");
                // Disconnect this websocket
                ctx.stop();
                return;
            }
            ctx.ping(b"ping");
        });
    }

    fn handle_message(&mut self, ctx: &mut ws::WebsocketContext<Self>, content: &str) {
        let message: WebsocketMessage = serde_json::from_str(&content).unwrap();
        match message {
            WebsocketMessage::Authentication { token } => {
                let claims = auth::Claims::from_token(&token).unwrap();
                log::info!("claims id {:?}", claims.id);
                self.id = Some(claims.id.clone());
                self.map.lock().unwrap().insert(claims.id, ctx.address());
                log::info!("{:?}", self.map);

                self.map
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|(k, _)| Some(*k) != self.id.as_ref())
                    .for_each(|(k, ws)| {
                        ws.try_send(WebsocketMessage::Hello { id: k.clone() })
                            .unwrap()
                    });
            }
            _ => (),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ProjectUpdateWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Pong(_)) => {
                log::info!("PONG!");
                self.hb = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(msg)) => {
                log::info!("Got a message from ws {:?}", msg);
                self.handle_message(ctx, &msg);
            }
            Err(e) => {
                log::error!("Error in message");
                panic!("{}", e)
            }
            _ => (),
        }
    }
}

impl Handler<WebsocketMessage> for ProjectUpdateWs {
    type Result = ();

    fn handle(&mut self, msg: WebsocketMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

pub async fn index(
    req: web::HttpRequest,
    state: web::Data<WebsocketState>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::error::Error> {
    let map = Arc::clone(&state.map);
    let resp = ws::start(ProjectUpdateWs::new(map), &req, stream);

    println!("{:?}", resp);
    resp
}
