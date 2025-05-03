use std::{
    io::BufRead,
    time::{Duration, UNIX_EPOCH},
};

use actix::prelude::*;
use actix_ogn::OGNMessage;
use chrono::prelude::*;
use itertools::Itertools;

pub struct StdinActor {
    pub recipient: Recipient<OGNMessage>,

    pub batch_size: usize,
}

impl StdinActor {
    pub fn new(recipient: Recipient<OGNMessage>, batch_size: usize) -> Self {
        StdinActor {
            recipient,
            batch_size,
        }
    }
}

impl Actor for StdinActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        println!("StdinActor started");

        for stdin_chunk_iter in std::io::stdin()
            .lock()
            .lines()
            .chunks(self.batch_size)
            .into_iter()
        {
            let batch: Vec<_> = stdin_chunk_iter
                .filter_map(|result| match result {
                    Ok(line) => match line.split_once(": ") {
                        Some((first, second)) => match first.parse::<u128>() {
                            Ok(nanos) => Some(OGNMessage {
                                raw: second.to_owned(),
                            }),
                            Err(err) => {
                                error!("{}: '{}'", err, line);
                                None
                            }
                        },
                        None => {
                            error!("Error splitting line: '{}'", line);
                            None
                        }
                    },
                    Err(err) => {
                        error!("Error reading from stdin: {}", err);
                        None
                    }
                })
                .collect();

            for container in batch {
                match self.recipient.do_send(container) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Error sending message: {}", err);
                    }
                }
            }
        }
    }
}
