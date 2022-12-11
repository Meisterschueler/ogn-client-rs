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
    pub distances: bool,
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
            if self.distances {
                ogn_packet.distance = ogn_packet
                    .aprs
                    .as_ref()
                    .ok()
                    .and_then(|aprs| self.distance_service.get_distance(aprs));
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
