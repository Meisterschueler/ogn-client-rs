extern crate actix;
extern crate actix_ogn;
extern crate aprs_parser;
extern crate pretty_env_logger;

mod console_logger;
mod ogn_comment;

use actix::*;
use actix_ogn::OGNActor;
use clap::Parser;
use console_logger::ConsoleLogger;
use ogn_comment::OGNComment;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Raw,
    Json,
    Influx,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
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

    let format = cli.format;

    let includes = cli
        .includes
        .and_then(|s| Some(s.split(',').map(|x| x.to_string()).collect::<Vec<_>>()));

    let excludes = cli
        .excludes
        .and_then(|s| Some(s.split(',').map(|x| x.to_string()).collect::<Vec<_>>()));

    // Start actix
    let sys = actix::System::new("test");

    // Start actor in separate threads
    let console_logger: Addr<_> = ConsoleLogger {
        format: format,
        includes: includes,
        excludes: excludes,
    }
    .start();

    // Start OGN client in separate thread
    let _addr: Addr<_> = Supervisor::start(move |_| OGNActor::new(console_logger.recipient()));

    let _result = sys.run();
}
