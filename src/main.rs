extern crate actix;
extern crate actix_ogn;
extern crate aprs_parser;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod date_time_guesser;
mod distance_service;
mod glidernet_collector;
mod ogn_comment;
mod ogn_packet;
mod output_handler;
mod receiver;

use actix::*;
use actix_ogn::OGNActor;
use clap::Parser;
use distance_service::DistanceService;
use glidernet_collector::GlidernetCollector;
use itertools::Itertools;
use ogn_comment::OGNComment;
use output_handler::OutputHandler;
use postgres::{Client, NoTls};
use receiver::Receiver;
use std::io::BufRead;

use crate::ogn_packet::OGNPacket;

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputSource {
    Glidernet,
    Stdin,
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Raw,
    Json,
    Influx,
    Csv,
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutputTarget {
    Stdout,
    PostgreSQL,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// specify input source
    #[arg(short, long, value_enum, default_value_t = InputSource::Glidernet)]
    source: InputSource,

    /// specify output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Raw)]
    format: OutputFormat,

    /// specify output target
    #[arg(short, long, value_enum, default_value_t = OutputTarget::Stdout)]
    target: OutputTarget,

    /// maximum batch size for parallel stdin execution
    #[arg(short, long, default_value = "16384")]
    batch_size: usize,

    /// calculate additional metrics like distance and normalized signal quality
    #[arg(short, long)]
    additional: bool,

    /// database connection string
    #[arg(
        short,
        long,
        default_value = "postgresql://postgres:postgres@localhost:5432/ogn"
    )]
    database_url: String,
}

fn main() {
    pretty_env_logger::init();

    // Get the command line arguments
    let cli = Cli::parse();

    let source = cli.source;
    let mut format = cli.format;
    let target = cli.target;
    let database_url = cli.database_url;
    let batch_size = cli.batch_size;

    match target {
        OutputTarget::Stdout => {
            if format == OutputFormat::Csv {
                println!("{}", OGNPacket::get_csv_header_positions());
            }
        }
        OutputTarget::PostgreSQL => match format {
            OutputFormat::Raw => {
                info!("Setting output format to CSV");
                format = OutputFormat::Csv;
            }
            _ => {
                error!("Output format is allowed for \"--target stdout\" only.");
                std::process::abort();
            }
        },
    }

    let mut output_handler = OutputHandler {
        target,
        format,
        client: if target == OutputTarget::PostgreSQL {
            let client = Client::connect(&database_url, NoTls).unwrap();
            Some(client)
        } else {
            None
        },
        distance_service: if cli.additional {
            Some(DistanceService::new())
        } else {
            None
        },
    };

    match source {
        InputSource::Stdin => {
            for stdin_chunk_iter in std::io::stdin()
                .lock()
                .lines()
                .chunks(batch_size)
                .into_iter()
            {
                let batch: Vec<(u128, String)> = stdin_chunk_iter
                    .filter_map(|result| match result {
                        Ok(line) => match line.split_once(": ") {
                            Some((first, second)) => match first.parse::<u128>() {
                                Ok(ts) => Some((ts, second.to_owned())),
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

                output_handler.parse(&batch);
            }
        }

        InputSource::Glidernet => {
            // Start actix
            let sys = actix::System::new("test");

            // Start actor in separate threads
            let glidernet_collector: Addr<_> = GlidernetCollector {
                output_handler,
                messages: vec![],
            }
            .start();

            // Start OGN client in separate thread
            let _addr: Addr<_> =
                Supervisor::start(move |_| OGNActor::new(glidernet_collector.recipient()));

            let _result = sys.run();
        }
    }
}
