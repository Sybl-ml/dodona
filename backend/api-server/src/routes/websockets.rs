//! Defines the websocket and related functions for realtime communication with the client

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, RwLock};

use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{stream_consumer::StreamConsumer, Consumer, DefaultConsumerContext};
use rdkafka::Message;
use tokio_stream::StreamExt;

use actix::prelude::Addr;
use actix::{Actor, ActorContext, AsyncContext, Handler, Running, StreamHandler};
use actix_web::{web, HttpResponse};
use actix_web_actors::ws;
use mongodb::bson::oid::ObjectId;
use std::time::{Duration, Instant};

use crate::{auth, routes::payloads::WebsocketMessage, WebsocketState};
use messages::KafkaWsMessage;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// ProjectUpdateWs struct
/// hb is the heartbeat to guarantee websocket is alive
/// id is the userid which opened this connection
/// map is the map of userids to websocket connections
#[derive(Debug)]
pub struct ProjectUpdateWs {
    hb: Instant,
    id: Option<ObjectId>,
    map: Arc<RwLock<HashMap<String, Addr<ProjectUpdateWs>>>>,
}

impl Actor for ProjectUpdateWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx)
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if let Some(id) = &self.id {
            self.map.write().unwrap().remove(&id.to_string());
        }
        Running::Stop
    }
}

impl ProjectUpdateWs {
    /// Creates a new ProjectUpdateWs
    pub fn new(map: Arc<RwLock<HashMap<String, Addr<ProjectUpdateWs>>>>) -> ProjectUpdateWs {
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
                self.map
                    .write()
                    .unwrap()
                    .insert(claims.id.to_string(), ctx.address());
                ctx.text(
                    serde_json::to_string(&WebsocketMessage::Hello { id: claims.id }).unwrap(),
                );
            }
            _ => (),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ProjectUpdateWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("Websocket connection closed");
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(msg)) => {
                log::debug!("message from ws client {:?}", msg);
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
        log::debug!("Sending msg to client {:?}", msg);
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

/// Index page for websocket connection
pub async fn index(
    req: web::HttpRequest,
    state: web::Data<WebsocketState>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::error::Error> {
    let map = Arc::clone(&state.map);
    let resp = ws::start(ProjectUpdateWs::new(map), &req, stream);
    resp
}

/// Consumes from kafka
pub async fn consume_updates(port: u16, map: Arc<RwLock<HashMap<String, Addr<ProjectUpdateWs>>>>) {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    log::info!("Broker Socket: {:?}", addr);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", "project_update")
        .set("bootstrap.servers", addr)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["project_updates"])
        .expect("Can't subscribe to project_updates");

    // Ignore any errors in the stream
    let mut message_stream = consumer.stream().filter_map(Result::ok);

    while let Some(message) = message_stream.next().await {
        // Interpret the content as a string
        let payload = match message.payload_view::<[u8]>() {
            // This cannot fail, `rdkafka` always returns `Ok(bytes)`
            Some(view) => view.unwrap(),
            None => {
                log::warn!("Received an empty message from Kafka");
                continue;
            }
        };

        log::debug!(
            "Message key: {:?}, Payload: {:?}, timestamp: {:?}",
            message.key(),
            &payload,
            message.timestamp()
        );

        let project_update: KafkaWsMessage<'_> = serde_json::from_slice(&payload).unwrap();
        let ws_msg = WebsocketMessage::from(&project_update);

        let user_id = std::str::from_utf8(&message.key().unwrap()).unwrap();
        let socket_map = map.read().unwrap();
        if let Some(socket) = socket_map.get(user_id) {
            socket.try_send(ws_msg).unwrap();
        }
    }
}
