use std::{collections::HashMap, time::SystemTime};

use actix::prelude::*;
use chrono::{DateTime, Utc};
use ogn_parser::{AprsData, AprsPosition, ServerResponse};

use crate::messages::server_response_container::ServerResponseContainer;

pub struct ValidationActor {
    pub recipient: Recipient<ServerResponseContainer>,

    pub reveivers_by_sender: HashMap<String, HashMap<String, (DateTime<Utc>, AprsPosition)>>,
    pub receivers: HashMap<String, (DateTime<Utc>, AprsPosition)>,
    pub last_server_timestamp: Option<DateTime<Utc>>,
}

impl ValidationActor {
    pub fn new(recipient: Recipient<ServerResponseContainer>) -> Self {
        ValidationActor {
            recipient,

            reveivers_by_sender: HashMap::new(),
            receivers: HashMap::new(),
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
                let sender_name = &packet.from.call;
                let receiver_name = &packet.via.iter().last().unwrap().call;
                if let AprsData::Position(position) = &packet.data {
                    // calculate absolute timestamp based on the relative timestamp and a reference time (from the server or localhost)
                    let timestamp_actual = if let Some(position_timestamp) = &position.timestamp {
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
                    server_response_containter.receiver_ts = timestamp_actual;

                    // calculate the distance and bearing from the receiver to the sender
                    if let Some((_, receiver)) = self.receivers.get_mut(receiver_name) {
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

                        server_response_containter.bearing = Some(bearing);
                        server_response_containter.distance = Some(distance);
                        server_response_containter.normalized_signal_quality =
                            normalized_signal_quality;
                    }

                    // calculate the plausibility of the message
                    // basics
                    // bit 0: timestamp is not plausible
                    //
                    // related to the receiver
                    // bit 1: no bearing or distance available (-> receiver not seen yet)
                    // bit 2: distance not plausible (> 1000km)
                    // bit 3: normalized_signal_quality not available
                    // bit 4: normalized_signal_quality not plausible (> 50dB@10km)
                    //
                    // related to the last message (same sender, same receiver)
                    // bit 5: last message not available
                    // bit 6: last message is not older than 300s
                    // bit 7: horizontal speed > 300m/s
                    // bit 8: vertical speed > 300ft/s
                    //
                    // // related to other receivers (same sender, other receiver)
                    // bit 9: sender has never been seen by another receiver
                    // bit 10: messages received by other receivers are older than 300s

                    let mut plausibility = 0;
                    if let Some(receiver_time_actual) = server_response_containter.receiver_ts {
                        if let (Some(_), Some(distance)) = (
                            server_response_containter.bearing,
                            server_response_containter.distance,
                        ) {
                            plausibility += if distance > 1000000.0 { 4 } else { 0 }; // distance > 1000km
                        } else {
                            plausibility += 2; // no bearing or distance
                        }

                        if let Some(normalized_signal_quality) =
                            server_response_containter.normalized_signal_quality
                        {
                            plausibility += if normalized_signal_quality > 50.0 {
                                // normalized_signal_quality > 50
                                16
                            } else {
                                0
                            };
                        } else {
                            plausibility += 8; // no normalized signal quality
                        }

                        if let Some(receivers) = self.reveivers_by_sender.get(sender_name) {
                            if let Some((receiver_time_previous, position_previous)) =
                                receivers.get(receiver_name)
                            {
                                let delta_seconds = receiver_time_actual
                                    .signed_duration_since(*receiver_time_previous)
                                    .num_seconds();

                                if delta_seconds <= 300 {
                                    let horizontal_speed =
                                        position.get_relation(position_previous).distance
                                            / delta_seconds as f64;
                                    if horizontal_speed > 300. {
                                        plausibility += 128; // horizontal speed > 300m/s
                                    }

                                    if let (Some(previous_altitude), Some(current_altitude)) = (
                                        position_previous.comment.altitude,
                                        position.comment.altitude,
                                    ) {
                                        let vertical_speed = (previous_altitude as f64
                                            - current_altitude as f64)
                                            / delta_seconds as f64;

                                        if vertical_speed > 300. {
                                            plausibility += 512; // vertical speed > 300ft/s
                                        }
                                    } else {
                                        plausibility += 256; // no altitude
                                    }
                                } else {
                                    plausibility += 64; // previous beacon too old
                                }
                            } else {
                                plausibility += 32; // no previous beacon with valid receiver_time from same sender
                            }

                            if self
                                .reveivers_by_sender
                                .iter()
                                .filter(|(other_receiver_name, _)| {
                                    *other_receiver_name != receiver_name
                                })
                                .count()
                                == 0
                            {
                                plausibility += 1024; // no other receivers
                            }

                            if receivers
                                .iter()
                                .filter(|(other_receiver_name, _)| {
                                    *other_receiver_name != receiver_name
                                })
                                .all(|(_, (receiver_time_other, _))| {
                                    receiver_time_other
                                        .signed_duration_since(receiver_time_actual)
                                        .num_seconds()
                                        > 300
                                })
                            {
                                plausibility += 2048; // other receivers are older than 300s
                            }
                        }
                    } else {
                        plausibility += 1; // no receiver time
                    }
                    server_response_containter.plausibility = Some(plausibility);

                    // store the beacon
                    if let Some(ts) = timestamp_actual {
                        self.reveivers_by_sender
                            .entry(sender_name.to_string())
                            .or_default()
                            .insert(receiver_name.to_string(), (ts, position.clone()));

                        self.receivers
                            .insert(sender_name.to_string(), (ts, position.clone()));
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
                println!("Error sending message: {err}");
            }
        }
    }
}
