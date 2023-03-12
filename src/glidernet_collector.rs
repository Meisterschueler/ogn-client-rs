use std::time::{Duration, SystemTime};

use actix::prelude::*;
use actix_ogn::OGNMessage;
use chrono::{DateTime, Utc};

use crate::output_handler::OutputHandler;

pub struct GlidernetCollector {
    pub messages: Vec<(DateTime<Utc>, String)>,
    pub output_handler: OutputHandler,
}

impl Actor for GlidernetCollector {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_secs(1), |act, _ctx| {
            act.output_handler.parse(&act.messages);
            act.messages.clear();
        });
    }
}

impl Handler<OGNMessage> for GlidernetCollector {
    type Result = ();

    fn handle(&mut self, msg: OGNMessage, _: &mut Context<Self>) {
        let ts = SystemTime::now().into();
        self.messages.push((ts, msg.raw));
    }
}
