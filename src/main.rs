extern crate actix;
extern crate actix_ogn;
extern crate aprs_parser;
extern crate pretty_env_logger;

mod console_logger;
mod ogn_comment;
mod ogn_message_converter;

use std::io::BufRead;
use std::io::Write;

use actix::*;
use actix_ogn::{OGNActor, OGNMessage};
use clap::Parser;
use console_logger::ConsoleLogger;
use ogn_comment::OGNComment;
use ogn_message_converter::OGNMessageConverter;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum InputSource {
    Glidernet,
    Console,
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

    let includes = cli
        .includes
        .and_then(|s| Some(s.split(',').map(|x| x.to_string()).collect::<Vec<_>>()));

    let excludes = cli
        .excludes
        .and_then(|s| Some(s.split(',').map(|x| x.to_string()).collect::<Vec<_>>()));

    match source {
        InputSource::Console => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
        
            for line in std::io::stdin().lock().lines() {
                let line = line.unwrap();
                let (first, second) = line.split_once(": ").unwrap();
                
                match first.parse::<u128>() {
                    Ok(ts) => {
                        let message = OGNMessage{raw: second.to_string()};
                        let result = match format {
                            OutputFormat::Raw => message.to_raw(ts),
                            OutputFormat::Json => message.to_json(ts),
                            OutputFormat::Influx => message.to_influx(ts),
                        };
                        write!(lock, "{result}").unwrap();
                    },
                    Err(err) => {
                        eprintln!("{err}: '{first}'. Complete string: \"{line}\"");
                    }
                }
            }
        },
        InputSource::Glidernet => {
            // Start actix
            let sys = actix::System::new("test");

            // Start actor in separate threads
            let console_logger: Addr<_> = ConsoleLogger {
                    source: source,
                    format: format,
                    includes: includes,
                    excludes: excludes,
                }
                .start();

            // Start OGN client in separate thread
            let _addr: Addr<_> = Supervisor::start(move |_| OGNActor::new(console_logger.recipient()));

            let _result = sys.run();
        }
    }
    
}
