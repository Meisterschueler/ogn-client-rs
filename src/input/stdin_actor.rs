use std::{
    io::BufRead,
    time::{Duration, UNIX_EPOCH},
};

use actix::prelude::*;
use chrono::prelude::*;
use itertools::Itertools;

use crate::messages::ognmessagewithtimestamp::OGNMessageWithTimestamp;

pub struct StdinActor {
    pub recipient: Recipient<OGNMessageWithTimestamp>,

    pub batch_size: usize,
}

impl StdinActor {
    pub fn new(recipient: Recipient<OGNMessageWithTimestamp>, batch_size: usize) -> Self {
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
                            Ok(nanos) => Some(OGNMessageWithTimestamp {
                                ts: DateTime::<Utc>::from(
                                    UNIX_EPOCH + Duration::from_nanos(nanos as u64),
                                ),
                                raw: second.to_owned(),
                            }),
                            Err(err) => {
                                error!("{err}: '{line}'");
                                None
                            }
                        },
                        None => {
                            error!("Error splitting line: '{line}'");
                            None
                        }
                    },
                    Err(err) => {
                        error!("Error reading from stdin: {err}");
                        None
                    }
                })
                .collect();

            for container in batch {
                match self.recipient.do_send(container) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Error sending message: {err}");
                    }
                }
            }
        }
    }
}
