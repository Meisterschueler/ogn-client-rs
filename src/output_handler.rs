use aprs_parser::AprsData;
use postgres::Client;
use rayon::prelude::*;

use crate::OutputTarget;
use crate::{distance_service::DistanceService, ogn_packet::OGNPacket, OutputFormat};
use std::io::Write;
pub struct OutputHandler {
    pub target: OutputTarget,
    pub format: OutputFormat,

    pub client: Option<Client>,
    pub distance_service: Option<DistanceService>,
}

impl OutputHandler {
    pub fn parse(&mut self, messages: &Vec<(u128, String)>) {
        // lines are parsed parallel
        let mut ogn_packets = messages
            .par_iter()
            .map(|(ts, line)| OGNPacket::new(*ts, line))
            .collect::<Vec<OGNPacket>>();

        // additional metrics are computed non-parallel
        if let Some(distance_service) = &mut self.distance_service {
            ogn_packets.iter_mut().for_each(|mut ogn_packet| {
                ogn_packet.distance = ogn_packet
                    .aprs
                    .as_ref()
                    .ok()
                    .and_then(|aprs| distance_service.get_distance(aprs));
                if let Some(distance) = ogn_packet.distance {
                    if let Some(comment) = &ogn_packet.comment {
                        if let Some(signal_quality) = comment.signal_quality {
                            ogn_packet.normalized_quality =
                                DistanceService::get_normalized_quality(distance, signal_quality);
                        }
                    }
                };
            });
        }

        // data rows are generated parallel
        let rows = ogn_packets
            .par_iter()
            .filter(|x| {
                if self.format == OutputFormat::Csv {
                    if x.aprs.is_ok() {
                        matches!(x.aprs.as_ref().unwrap().data, AprsData::Position(_))
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .map(|ogn_packet| match self.format {
                OutputFormat::Raw => ogn_packet.to_raw(),
                OutputFormat::Json => ogn_packet.to_json(),
                OutputFormat::Influx => ogn_packet.to_influx(),
                OutputFormat::Csv => ogn_packet.to_csv(),
            })
            .collect::<Vec<String>>();

        // generate output
        let sep = match self.format {
            OutputFormat::Raw => "\n",
            OutputFormat::Json => ",",
            OutputFormat::Influx => "",
            OutputFormat::Csv => "\n",
        };
        match self.target {
            OutputTarget::Stdout => {
                let stdout = std::io::stdout();
                let mut lock = stdout.lock();
                for line in rows {
                    write!(lock, "{line}{sep}").unwrap();
                }
            }
            OutputTarget::PostgreSQL => {
                let sql = format!(
                    "COPY positions ({}) FROM STDIN WITH (FORMAT CSV)",
                    OGNPacket::get_csv_header_positions()
                );

                let client = self.client.as_mut().unwrap();
                let mut copy_stdin = client.copy_in(&sql).unwrap();
                copy_stdin.write_all(rows.join(sep).as_bytes()).unwrap();
                copy_stdin.finish().unwrap();

                info!("{} points inserted", rows.len());
            }
        }
    }
}
