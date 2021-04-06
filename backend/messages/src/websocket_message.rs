use actix::prelude::{Message, Recipient};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientCompleteMessage{
    model_id: String,
    model_count: usize,
  }