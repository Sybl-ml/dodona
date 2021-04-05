use actix::prelude::{Message, Recipient};

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCompleteMessage<'a> {
    model_id: &'a str,
    model_count: usize,
  }