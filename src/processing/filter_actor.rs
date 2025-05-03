use std::collections::HashSet;

use actix::prelude::*;
use ogn_parser::ServerResponse;

use crate::messages::server_response_container::ServerResponseContainer;

pub struct FilterActor {
    pub recipient: Recipient<ServerResponseContainer>,

    pub include: Option<HashSet<String>>,
    pub exclude: Option<HashSet<String>>,
}

impl FilterActor {
    pub fn new(
        recipient: Recipient<ServerResponseContainer>,
        include: Option<HashSet<String>>,
        exclude: Option<HashSet<String>>,
    ) -> Self {
        FilterActor {
            recipient,
            include,
            exclude,
        }
    }
}

impl Actor for FilterActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("FilterActor started");
    }
}

impl Handler<ServerResponseContainer> for FilterActor {
    type Result = ();

    fn handle(&mut self, msg: ServerResponseContainer, _: &mut Self::Context) {
        if let ServerResponse::AprsPacket(packet) = &msg.server_response {
            if let Some(include) = &self.include {
                if !include.contains(&packet.to.to_string()) {
                    return;
                }
            }

            if let Some(exclude) = &self.exclude {
                if exclude.contains(&packet.to.to_string()) {
                    return;
                }
            }
        }

        // Forward the message to the next actor in the chain
        match self.recipient.do_send(msg) {
            Ok(_) => (),
            Err(err) => {
                error!("Error sending message to recipient: {}", err);
            }
        }
    }
}
