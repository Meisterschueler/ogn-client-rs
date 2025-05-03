extern crate actix;
extern crate actix_ogn;
extern crate ogn_parser;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod element_getter;
mod input;
mod output;
mod processing;
mod server_response_container;

use actix::*;
use actix_ogn::OGNActor;
use clap::Parser;
use input::stdin_actor::StdinActor;
use output::influxdb_actor::InfluxDBActor;
use output::postgresql_actor::PostgreSQLActor;
use output::stdout_actor::StdoutActor;
use processing::filter_actor::FilterActor;
use processing::parser_actor::ParserActor;
use processing::validation_actor::ValidationActor;
use std::collections::HashSet;

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputSource {
    Glidernet,
    Stdin,
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutputTarget {
    Stdout,
    PostgreSQL,
    InfluxDB,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// specify input source
    #[arg(short, long, value_enum, default_value_t = InputSource::Glidernet)]
    source: InputSource,

    /// specify output target
    #[arg(short, long, value_enum, default_value_t = OutputTarget::Stdout)]
    target: OutputTarget,

    /// maximum batch size for parallel stdin execution
    #[arg(short, long, default_value = "16384")]
    batch_size: usize,

    /// database connection string
    #[arg(
        short,
        long,
        default_value = "postgresql://postgres:postgres@localhost:5432/ogn"
    )]
    database_url: String,

    /// let pass only packets with given destination callsigns (comma separated)
    #[arg(short, long)]
    included: Option<String>,

    /// drop packets with given destination callsigns (comma separated)
    #[arg(short, long)]
    excluded: Option<String>,
}

fn main() {
    pretty_env_logger::init();

    // Get the command line arguments
    let cli = Cli::parse();

    let source = cli.source;
    let target = cli.target;
    let database_url = cli.database_url;
    let batch_size = cli.batch_size;
    let included = cli.included.map(|s| {
        s.split(",")
            .map(|s| s.to_string())
            .collect::<HashSet<String>>()
    });
    let excluded = cli.excluded.map(|s| {
        s.split(",")
            .map(|s| s.to_string())
            .collect::<HashSet<String>>()
    });

    // The pipeline is as follows:
    // 1. Input source (yields raw OGN messages)
    // 2. Parser actor (yields parsed data)
    // 3. Filter actor (filters the parsed data based on included/excluded destination callsigns)
    // 4. Validation actor (calculates additional data (e.g. distance, bearing, ...) and validates the parsed data)
    // 5. Output target (writes the data to the chosen output target)

    // Start actix
    let sys = actix::System::new("test");

    // Connect the chosen output actor with the validation actor
    let validator = match target {
        OutputTarget::Stdout => {
            let stdout = StdoutActor::new().start();
            ValidationActor::new(stdout.recipient()).start()
        }
        OutputTarget::PostgreSQL => {
            let postgresql = PostgreSQLActor::new(&database_url).start();
            ValidationActor::new(postgresql.recipient()).start()
        }
        OutputTarget::InfluxDB => {
            let influxdb = InfluxDBActor::new().start();
            ValidationActor::new(influxdb.recipient()).start()
        }
    };

    // Connect the validation actor to the filter actor
    let filter = FilterActor::new(validator.recipient(), included, excluded).start();

    // Connect the filter actor to the parser actor
    let parser = ParserActor::new(filter.recipient()).start();

    // Connect the parser actor to the input actor
    match source {
        InputSource::Glidernet => {
            // Glidernet can crash, so we use a supervisor
            Supervisor::start(move |_| OGNActor::new(parser.recipient()));
        }
        InputSource::Stdin => {
            StdinActor::new(parser.recipient(), batch_size).start();
        }
    };

    let _result = sys.run();
}
