extern crate actix;
extern crate actix_ogn;
extern crate pretty_env_logger;

extern crate json_patch;

use actix::*;
use actix_ogn::OGNMessage;
use std::time::SystemTime;

use crate::{InputSource, OutputFormat, OGNMessageConverter};


pub struct ConsoleLogger {
    pub source: InputSource,
    pub format: OutputFormat,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
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
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let output_string =  match self.format {
                OutputFormat::Raw => message.to_raw(timestamp),
                OutputFormat::Json => message.to_json(timestamp),
                OutputFormat::Influx => message.to_influx(timestamp),
            };
            println!("{output_string}");
        }
    }
}
