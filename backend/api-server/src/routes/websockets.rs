use actix::{Actor, StreamHandler}
use actix_web::{web, HttpResponse};
use actix_web_actors::ws;

// TODO: Add a userId to this struct for a bit of state
// using this userid it will be possible to subscribe to the correct topic
// again possibly using the new datastructure that is storing the topic names

struct ProjectUpdateWs;

impl Actor for ProjectUpdateWs {
    type Context = ws::WebsocketContext<Self>;
}

// TODO: In Handle continously loop on the topic waiting for a update
// If update send a message using the context across the websocket.
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ProjectUpdateWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg)
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(ProjectUpdateWs {}, &req, stream);
    println!("{:?}", resp);
    resp
}