use std::{collections::HashMap, time::SystemTime};

use actix::prelude::*;
use chrono::{DateTime, Utc};
use ogn_parser::{AprsData, AprsPosition, ServerResponse};

use crate::messages::server_response_container::ServerResponseContainer;

pub struct ValidationActor {
    pub recipient: Recipient<ServerResponseContainer>,

    pub positions: HashMap<String, AprsPosition>,
    pub last_server_timestamp: Option<DateTime<Utc>>,
}

impl ValidationActor {
    pub fn new(recipient: Recipient<ServerResponseContainer>) -> Self {
        ValidationActor {
            recipient,

            positions: HashMap::new(),
            last_server_timestamp: None,
        }
    }
}

impl Actor for ValidationActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("ValidationActor started");
    }
}

impl Handler<ServerResponseContainer> for ValidationActor {
    type Result = ();

    fn handle(
        &mut self,
        mut server_response_containter: ServerResponseContainer,
        _: &mut Context<Self>,
    ) {
        let use_server_timestamp = false; // actix_ogn does not support server timestamp yet (included in the server comment)

        match &server_response_containter.server_response {
            ServerResponse::AprsPacket(packet) => {
                if let AprsData::Position(position) = &packet.data {
                    let timestamp_validated = if let Some(position_timestamp) = &position.timestamp
                    {
                        if use_server_timestamp {
                            self.last_server_timestamp.and_then(|reference| {
                                position_timestamp.to_datetime(&reference).ok()
                            })
                        } else {
                            let reference = DateTime::<Utc>::from(SystemTime::now());
                            position_timestamp.to_datetime(&reference).ok()
                        }
                    } else {
                        None
                    };

                    let receiver_name = &packet.via.iter().last().unwrap().call;
                    if let Some(receiver) = self.positions.get_mut(receiver_name) {
                        let relation = receiver.get_relation(position);

                        let bearing = relation.bearing;
                        let distance = relation.distance;

                        let normalized_signal_quality =
                            position.comment.signal_quality.and_then(|signal_quality| {
                                if signal_quality > 0.0 {
                                    Some(
                                        signal_quality as f64
                                            + 20.0 * (distance / 10_000.0).log10(),
                                    )
                                } else {
                                    None
                                }
                            });

                        server_response_containter.receiver_time = timestamp_validated;
                        server_response_containter.bearing = Some(bearing);
                        server_response_containter.distance = Some(distance);
                        server_response_containter.normalized_signal_quality =
                            normalized_signal_quality;
                    } else {
                        self.positions
                            .insert(receiver_name.to_string(), position.clone());
                    }
                }
            }
            ServerResponse::ServerComment(server_comment) => {
                self.last_server_timestamp = Some(server_comment.timestamp);
            }
            ServerResponse::ParserError(_) => {}
        };

        match self.recipient.do_send(server_response_containter) {
            Ok(_) => {}
            Err(err) => {
                println!("Error sending message: {}", err);
            }
        }
    }
}
