use actix::prelude::*;

use crate::messages::server_response_container::ServerResponseContainer;

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
        if let Some(nanos) = msg.ts.timestamp_nanos_opt() {
            println!("{}: {}", nanos, msg.raw_message);
        } else {
            error!("Invalid timestamp: {}", msg.raw_message);
        }
    }
}
