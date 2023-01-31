extern crate actix;
extern crate actix_ogn;
extern crate aprs_parser;
extern crate pretty_env_logger;

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

    /// calculate additional metrics like distance and normalized signal quality
    #[arg(short, long)]
    additional: bool,

    /// proceed only APRS messages including a substring - format: comma separated strings
    #[arg(short, long)]
    includes: Option<String>,

    /// don't proceed APRS messages including a substring - format: comma separated strings
    #[arg(short, long)]
    excludes: Option<String>,
}

fn main() {
    pretty_env_logger::init();

    // Get the command line arguments
    let cli = Cli::parse();

    let source = cli.source;
    let format = cli.format;
    let target = cli.target;

    let includes = cli
        .includes
        .map(|s| s.split(',').map(|x| x.to_string()).collect::<Vec<_>>());

    let excludes = cli
        .excludes
        .map(|s| s.split(',').map(|x| x.to_string()).collect::<Vec<_>>());

    let mut output_handler = OutputHandler {
        target,
        format,
        includes,
        excludes,
        client: if target == OutputTarget::PostgreSQL {
            let url = "postgresql://postgres:changeme@localhost:5432/ogn";
            let client = Client::connect(url, NoTls).unwrap();
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
            for stdin_chunk_iter in std::io::stdin().lock().lines().chunks(16384).into_iter() {
                let batch: Vec<(u128, String)> = stdin_chunk_iter
                    .filter_map(|result| match result {
                        Ok(line) => {
                            let (first, second) = line.split_once(": ").unwrap();
                            match first.parse::<u128>() {
                                Ok(ts) => Some((ts, second.to_owned())),
                                Err(err) => {
                                    eprintln!("Error parsing timestamp: {}", err);
                                    None
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error reading from stdin: {}", err);
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
