extern crate actix;
extern crate actix_ogn;
extern crate aprs_parser;
extern crate pretty_env_logger;

mod console_logger;
mod distance_service;
mod ogn_comment;
mod ogn_packet;
mod receiver;

use std::io::BufRead;
use std::io::Write;

use actix::*;
use actix_ogn::OGNActor;
use clap::Parser;
use console_logger::ConsoleLogger;
use distance_service::DistanceService;
use ogn_comment::OGNComment;
use ogn_packet::OGNPacket;
use rayon::prelude::*;
use receiver::Receiver;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum InputSource {
    Glidernet,
    Stdin,
    StdinParallel,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Raw,
    Json,
    Influx,
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
    let mut distance_service = DistanceService::new();

    // Get the command line arguments
    let cli = Cli::parse();

    let source = cli.source;
    let format = cli.format;
    let additional = cli.additional;

    let includes = cli
        .includes
        .and_then(|s| Some(s.split(',').map(|x| x.to_string()).collect::<Vec<_>>()));

    let excludes = cli
        .excludes
        .and_then(|s| Some(s.split(',').map(|x| x.to_string()).collect::<Vec<_>>()));

    match source {
        InputSource::StdinParallel => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();

            let mut line_iter = std::io::stdin().lock().lines();
            loop {
                let mut batch = vec![];
                for _ in 0..16384 {
                    if let Some(line) = line_iter.next() {
                        match line {
                            Ok(line) => {
                                batch.push(line);
                            }
                            Err(_) => {
                                eprintln!("WTF")
                            }
                        }
                    } else {
                        break;
                    }
                }
                if batch.is_empty() {
                    break;
                }

                let out_lines: Vec<_> = if additional {
                    let ogn_packets = batch
                        .par_iter()
                        .filter_map(|line| match line.parse::<OGNPacket>() {
                            Ok(ogn_packet) => Some(ogn_packet),
                            Err(_) => {
                                eprintln!("Complete string: \"{line}\"");
                                None
                            }
                        })
                        .collect::<Vec<OGNPacket>>();

                    ogn_packets
                        .into_iter()
                        .map(|mut ogn_packet| {
                            ogn_packet.distance = ogn_packet
                                .aprs
                                .as_ref()
                                .ok()
                                .and_then(|aprs| distance_service.get_distance(aprs));
                            if let Some(distance) = ogn_packet.distance {
                                if let Some(comment) = &ogn_packet.comment {
                                    if let Some(signal_quality) = comment.signal_quality {
                                        ogn_packet.normalized_quality =
                                            DistanceService::get_normalized_quality(
                                                distance,
                                                signal_quality,
                                            );
                                    }
                                }
                            };
                            match format {
                                OutputFormat::Raw => ogn_packet.to_raw(),
                                OutputFormat::Json => ogn_packet.to_json(),
                                OutputFormat::Influx => ogn_packet.to_influx(),
                            }
                        })
                        .collect::<Vec<String>>()
                } else {
                    batch
                        .par_iter()
                        .map(|line| match line.parse::<OGNPacket>() {
                            Ok(ogn_packet) => match format {
                                OutputFormat::Raw => ogn_packet.to_raw(),
                                OutputFormat::Json => ogn_packet.to_json(),
                                OutputFormat::Influx => ogn_packet.to_influx(),
                            },
                            Err(_) => {
                                eprintln!("Complete string: \"{line}\"");
                                String::new()
                            }
                        })
                        .collect::<Vec<String>>()
                };

                for line in out_lines {
                    write!(lock, "{line}").unwrap();
                }
            }
        }
        InputSource::Stdin => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();

            for line in std::io::stdin().lock().lines() {
                match line {
                    Ok(line) => match line.parse::<OGNPacket>() {
                        Ok(mut ogn_packet) => {
                            if additional {
                                ogn_packet.distance = ogn_packet
                                    .aprs
                                    .as_ref()
                                    .ok()
                                    .and_then(|aprs| distance_service.get_distance(aprs));
                                if let Some(distance) = ogn_packet.distance {
                                    if let Some(comment) = &ogn_packet.comment {
                                        if let Some(signal_quality) = comment.signal_quality {
                                            ogn_packet.normalized_quality =
                                                DistanceService::get_normalized_quality(
                                                    distance,
                                                    signal_quality,
                                                );
                                        }
                                    }
                                }
                            };
                            let result = match format {
                                OutputFormat::Raw => ogn_packet.to_raw(),
                                OutputFormat::Json => ogn_packet.to_json(),
                                OutputFormat::Influx => ogn_packet.to_influx(),
                            };
                            write!(lock, "{result}").unwrap();
                        }
                        Err(_) => {
                            eprintln!("Complete string: \"{line}\"");
                        }
                    },
                    Err(_) => eprintln!("IO error"),
                }
            }
        }
        InputSource::Glidernet => {
            // Start actix
            let sys = actix::System::new("test");

            // Start actor in separate threads
            let console_logger: Addr<_> = ConsoleLogger {
                source: source,
                format: format,
                distances: additional,
                includes: includes,
                excludes: excludes,

                distance_service: distance_service,
            }
            .start();

            // Start OGN client in separate thread
            let _addr: Addr<_> =
                Supervisor::start(move |_| OGNActor::new(console_logger.recipient()));

            let _result = sys.run();
        }
    }
}
