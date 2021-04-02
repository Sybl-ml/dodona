use actix::{Actor, StreamHandler}
use actix_web::{web, HttpResponse};
use actix_web_actors::ws;

struct ProjectUpdateWs;

impl Actor for ProjectUpdateWs {
    type Context = ws::WebsocketContext<Self>;
}

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
    let resp = ws::start(MyWs {}, &req, stream);
    println!("{:?}", resp);
    resp
}