extern crate actix;
extern crate actix_ogn;
extern crate pretty_env_logger;

extern crate json_patch;

use actix::*;
use actix_ogn::OGNMessage;
use std::time::SystemTime;

use crate::{DistanceService, InputSource, OGNPacket, OutputFormat};

pub struct ConsoleLogger {
    pub source: InputSource,
    pub format: OutputFormat,
    pub additional: bool,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,

    pub distance_service: DistanceService,
}

impl Actor for ConsoleLogger {
    type Context = Context<Self>;
}

impl Handler<OGNMessage> for ConsoleLogger {
    type Result = ();
    fn handle(&mut self, message: OGNMessage, _: &mut Context<Self>) {
        let passes_filter = if let Some(includes) = &self.includes {
            includes
                .iter()
                .any(|substring| message.raw.contains(substring))
        } else if let Some(excludes) = &self.excludes {
            !excludes
                .iter()
                .any(|substring| message.raw.contains(substring))
        } else {
            true
        };

        if passes_filter {
            let ts = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let mut ogn_packet = OGNPacket::new(ts, &message.raw);
            if self.additional {
                ogn_packet.distance = ogn_packet
                    .aprs
                    .as_ref()
                    .ok()
                    .and_then(|aprs| self.distance_service.get_distance(aprs));
                if let Some(distance) = ogn_packet.distance {
                    if let Some(comment) = &ogn_packet.comment {
                        if let Some(signal_quality) = comment.signal_quality {
                            ogn_packet.normalized_quality =
                                DistanceService::get_normalized_quality(distance, signal_quality);
                        }
                    }
                }
            };
            let output_string = match self.format {
                OutputFormat::Raw => ogn_packet.to_raw(),
                OutputFormat::Json => ogn_packet.to_json(),
                OutputFormat::Influx => ogn_packet.to_influx(),
            };
            println!("{output_string}");
        }
    }
}
