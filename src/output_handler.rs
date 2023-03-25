use chrono::{DateTime, Utc};
use postgres::Client;
use rayon::prelude::*;
use std::io::Write;

use crate::ogn_packet::{
    CsvSerializer, OGNPacketInvalid, OGNPacketPosition, OGNPacketStatus, OGNPacketUnknown,
};
use crate::OutputTarget;
use crate::{distance_service::DistanceService, ogn_packet::OGNPacket, OutputFormat};

pub struct OutputHandler {
    pub target: OutputTarget,
    pub format: OutputFormat,

    pub client: Option<Client>,
    pub distance_service: DistanceService,
}

impl OutputHandler {
    pub fn parse(&mut self, messages: &Vec<(DateTime<Utc>, String)>) {
        // lines are parsed parallel
        let mut packets = messages
            .par_iter()
            .map(|(ts, line)| OGNPacket::new(*ts, line))
            .collect::<Vec<OGNPacket>>();

        // additional metrics are computed non-parallel
        packets.iter_mut().for_each(|packet| {
            if let OGNPacket::Position(packet) = packet {
                packet.relation = self.distance_service.get_relation(packet);
                if let (Some(relation), Some(signal_quality)) =
                    (packet.relation, packet.comment.signal_quality)
                {
                    packet.normalized_quality = DistanceService::get_normalized_quality(
                        relation.distance,
                        signal_quality.into(),
                    );
                }
            };
        });

        match self.target {
            OutputTarget::Stdout => {
                let rows = match self.format {
                    OutputFormat::Raw => todo!(),
                    OutputFormat::Json => todo!(),
                    OutputFormat::Influx => packets
                        .par_iter()
                        .map(|p| match p {
                            OGNPacket::Invalid(inv) => OGNPacketInvalid::to_ilp(
                                "invalids",
                                inv.get_tags(),
                                inv.get_fields(),
                                inv.ts,
                            ),
                            OGNPacket::Unknown(unk) => OGNPacketUnknown::to_ilp(
                                "unknowns",
                                unk.get_tags(),
                                unk.get_fields(),
                                unk.ts,
                            ),
                            OGNPacket::Position(pos) => OGNPacketPosition::to_ilp(
                                "positions",
                                pos.get_tags(),
                                pos.get_fields(),
                                pos.ts,
                            ),
                            OGNPacket::Status(sta) => OGNPacketStatus::to_ilp(
                                "statuses",
                                sta.get_tags(),
                                sta.get_fields(),
                                sta.ts,
                            ),
                        })
                        .collect::<Vec<_>>(),
                    OutputFormat::Csv => todo!(),
                };
                println!("{}", rows.join(""));
            }
            OutputTarget::PostgreSQL => {
                let mut invalids: Vec<OGNPacketInvalid> = vec![];
                let mut unknowns: Vec<OGNPacketUnknown> = vec![];
                let mut positions: Vec<OGNPacketPosition> = vec![];
                let mut statuses: Vec<OGNPacketStatus> = vec![];

                for packet in packets {
                    match packet {
                        OGNPacket::Invalid(p) => invalids.push(p),
                        OGNPacket::Unknown(p) => unknowns.push(p),
                        OGNPacket::Position(p) => positions.push(p),
                        OGNPacket::Status(p) => statuses.push(p),
                    }
                }

                let sql_invalid_rows = invalids.par_iter().map(|p| p.to_csv()).collect::<Vec<_>>();
                let sql_unknown_rows = unknowns.par_iter().map(|p| p.to_csv()).collect::<Vec<_>>();
                let sql_position_rows =
                    positions.par_iter().map(|p| p.to_csv()).collect::<Vec<_>>();
                let sql_status_rows = statuses.par_iter().map(|p| p.to_csv()).collect::<Vec<_>>();

                self.insert_into_db(
                    "invalids",
                    &OGNPacketInvalid::csv_header(),
                    sql_invalid_rows,
                );
                self.insert_into_db(
                    "unknowns",
                    &OGNPacketUnknown::csv_header(),
                    sql_unknown_rows,
                );
                self.insert_into_db(
                    "positions",
                    &OGNPacketPosition::csv_header(),
                    sql_position_rows,
                );
                self.insert_into_db("statuses", &OGNPacketStatus::csv_header(), sql_status_rows);
            }
        }
    }

    fn insert_into_db(&mut self, table_name: &str, csv_header: &str, rows: Vec<String>) {
        let client = self.client.as_mut().unwrap();
        let sql_header = format!("COPY {table_name} ({csv_header}) FROM STDIN WITH (FORMAT CSV)");
        let mut copy_stdin = client.copy_in(&sql_header).unwrap();
        copy_stdin.write_all(rows.join("\n").as_bytes()).unwrap();
        match copy_stdin.finish() {
            Ok(_) => info!(
                "{} messages inserted into table '{}'",
                rows.len(),
                table_name
            ),
            Err(err) => error!(
                "Error: {}\nTable: {}\nRows: {}",
                err.to_string(),
                table_name,
                rows.join("\n")
            ),
        };
    }
}
