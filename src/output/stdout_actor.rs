use actix::prelude::*;

use crate::server_response_container::ServerResponseContainer;

pub struct StdoutActor;

impl StdoutActor {
    pub fn new() -> Self {
        StdoutActor
    }
}

impl Actor for StdoutActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("StdoutActor started");
    }
}

impl Handler<ServerResponseContainer> for StdoutActor {
    type Result = ();

    fn handle(&mut self, msg: ServerResponseContainer, _: &mut Self::Context) {
        if msg.normalized_signal_quality.is_some() {
            println!("Received message: {:?}", msg.raw_message);
            println!(
                "{:#?} {:#?} {:#?}",
                msg.distance, msg.bearing, msg.normalized_signal_quality
            );
        }
    }
}
