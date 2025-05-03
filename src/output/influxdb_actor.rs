use actix::prelude::*;

use crate::server_response_container::ServerResponseContainer;

pub struct InfluxDBActor;

impl Actor for InfluxDBActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("InfluxDBActor started");
    }
}

impl InfluxDBActor {
    pub fn new() -> Self {
        InfluxDBActor
    }
}

impl Handler<ServerResponseContainer> for InfluxDBActor {
    type Result = ();

    fn handle(&mut self, msg: ServerResponseContainer, _: &mut Self::Context) {
        println!("Received message: {:?}", msg.to_ilp());
    }
}
