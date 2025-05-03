use std::time::SystemTime;

use actix::prelude::*;
use actix_ogn::OGNMessage;
use chrono::{DateTime, Utc};
use ogn_parser::ServerResponse;

use crate::messages::{
    ognmessagewithtimestamp::OGNMessageWithTimestamp,
    server_response_container::ServerResponseContainer,
};

pub struct ParserActor {
    pub recipient: Recipient<ServerResponseContainer>,
}

impl ParserActor {
    pub fn new(recipient: Recipient<ServerResponseContainer>) -> Self {
        ParserActor { recipient }
    }
}

impl Actor for ParserActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("ParserActor started");
    }
}

impl Handler<OGNMessage> for ParserActor {
    type Result = ();

    fn handle(&mut self, msg: OGNMessage, _: &mut Context<Self>) {
        let ts: DateTime<Utc> = SystemTime::now().into();
        let server_response = msg.raw.parse::<ServerResponse>().unwrap();
        let server_response_containter = ServerResponseContainer {
            ts,
            raw_message: msg.raw.to_owned(),
            server_response,
            receiver_time: None,
            bearing: None,
            distance: None,
            normalized_signal_quality: None,
        };

        match self.recipient.do_send(server_response_containter) {
            Ok(_) => {}
            Err(err) => {
                println!("Error sending message: {}", err);
            }
        }
    }
}

impl Handler<OGNMessageWithTimestamp> for ParserActor {
    type Result = ();

    fn handle(&mut self, msg: OGNMessageWithTimestamp, _: &mut Context<Self>) {
        let server_response = msg.raw.parse::<ServerResponse>().unwrap();
        let server_response_containter = ServerResponseContainer {
            ts: msg.ts,
            raw_message: msg.raw.to_owned(),
            server_response,
            receiver_time: None,
            bearing: None,
            distance: None,
            normalized_signal_quality: None,
        };

        match self.recipient.do_send(server_response_containter) {
            Ok(_) => {}
            Err(err) => {
                println!("Error sending message: {}", err);
            }
        }
    }
}
