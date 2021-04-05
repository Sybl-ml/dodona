use actix::{Actor, StreamHandler}
use actix_web::{web, HttpResponse};
use actix_web_actors::ws;
use mongodb::bson::oid::ObjectId

// TODO: Add a userId to this struct for a bit of state
// using this userid it will be possible to subscribe to the correct topic
// again possibly using the new datastructure that is storing the topic names
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct ProjectUpdateWs {
    hb: Instant,
    id: ObjectId
};

ProjectUpdateWs {
    pub fn new(user_id: ObjectId) -> ProjectUpdateWs {
        ProjectUpdateWs {
            id: ObjectId,
            hb: Instant::now()
        }
    }
}

impl Actor for ProjectUpdateWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx)
    }
}

impl ProjectUpdateWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::info("Disconnecting failed heartbeat");
                // Disconnect this websocket
                ctx.stop();
                return;
            }

            ctx.ping(b"ping");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ProjectUpdateWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => self.hb = Instant::now
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();  
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(msg)) => {
                log::info("Got a message from ws {:?}", msg);
                ctx.send(msg.0);
            }
            Err(e) => {
                log::error("Error in message");
                panic!(e)
            }
        }
    }
}

async fn index(
    claims: auth::Claims,
    req: HttpRequest,
    state: web::Data<State>,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {

    let resp = ws::start(ProjectUpdateWs::new(claims.id)}, &req, stream);

    // Add to hashmap

    println!("{:?}", resp);
    resp
}