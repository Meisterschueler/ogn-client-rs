use std::time::{Duration, SystemTime};

use actix::prelude::*;
use actix_ogn::OGNMessage;

use crate::output_handler::OutputHandler;

pub struct GlidernetCollector {
    pub messages: Vec<(u128, String)>,
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
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        self.messages.push((ts, msg.raw));
    }
}
