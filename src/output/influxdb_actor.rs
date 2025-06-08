use actix::prelude::*;

use crate::{
    containers::containers::Container, messages::server_response_container::ServerResponseContainer,
};

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
        let container = msg.into();
        match container {
            Container::Position(position) => {
                println!("{}", position.to_ilp());
            }
            Container::Status(status) => {
                println!("{}", status.to_ilp());
            }
            _ => {
                // For now, just print the message
                //println!("Received container: {:?}", container);
            }
        }
    }
}
