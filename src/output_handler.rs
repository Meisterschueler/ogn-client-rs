use chrono::{DateTime, Utc};
use ogn_parser::{AprsData, AprsPosition, ServerResponse};
use postgres::Client;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Write;

use crate::OutputFormat;
use crate::OutputTarget;
use crate::ServerResponseContainer;

pub struct OutputHandler {
    pub target: OutputTarget,
    pub format: OutputFormat,

    pub client: Option<Client>,
    pub positions: HashMap<String, AprsPosition>,
    pub last_server_timestamp: Option<DateTime<Utc>>,
}

impl OutputHandler {
    pub fn parse(&mut self, messages: &Vec<(DateTime<Utc>, String)>) {
        // parallel: parse messages and set their timestamps and raw strings
        let mut containers = messages
            .par_iter()
            .map(|(ts, line)| {
                let server_response = line.parse::<ServerResponse>().unwrap();
                ServerResponseContainer {
                    ts: *ts,
                    raw_message: line.to_owned(),
                    server_response,
                    validated_timestamp: None,
                    bearing: None,
                    distance: None,
                    normalized_signal_quality: None,
                }
            })
            .collect::<Vec<_>>();

        // non-parallel: compute additional metrics (related to other messages) and validate the timestamp (related to the server timestamp)
        containers
            .iter_mut()
            .for_each(|container| match &container.server_response {
                ServerResponse::AprsPacket(packet) => {
                    if let AprsData::Position(position) = &packet.data {
                        let timestamp_validated =
                            if let (Some(server_timestamp), Some(position_timestamp)) =
                                (self.last_server_timestamp, position.timestamp.clone())
                            {
                                position_timestamp.to_datetime(&server_timestamp).ok()
                            } else {
                                None
                            };

                        let receiver_name = &packet.via.iter().last().unwrap().call;
                        if let Some(receiver) = self.positions.get_mut(receiver_name) {
                            let bearing = (receiver.get_bearing(position) + 360.0) % 360.0;
                            let distance = receiver.get_distance(position) * 1000.0;

                            let normalized_signal_quality =
                                position.comment.signal_quality.and_then(|signal_quality| {
                                    if signal_quality > 0.0 {
                                        Some(
                                            signal_quality as f64
                                                + 20.0 * (distance / 10_000.0).log10(),
                                        )
                                    } else {
                                        None
                                    }
                                });

                            container.validated_timestamp = timestamp_validated;
                            container.bearing = Some(bearing);
                            container.distance = Some(distance);
                            container.normalized_signal_quality = normalized_signal_quality;
                        } else {
                            self.positions
                                .insert(receiver_name.to_string(), position.clone());
                        }
                    }
                }
                ServerResponse::ServerComment(server_comment) => {
                    self.last_server_timestamp = Some(server_comment.timestamp);
                }
                ServerResponse::ParserError(parser_error) => {}
            });

        match self.target {
            OutputTarget::Stdout => {
                let rows: Vec<String>;
                let separator: String;
                match self.format {
                    OutputFormat::Raw => {
                        rows = containers
                            .par_iter()
                            //.map(|server_response| server_response.raw_message.to_string())
                            .map(|_| "WTF".to_string())
                            .collect();
                        separator = "\n".to_string();
                    }
                    OutputFormat::Json => todo!(),
                    OutputFormat::Influx => {
                        rows = containers
                            .par_iter()
                            .map(|p| p.to_ilp())
                            .collect::<Vec<_>>();
                        separator = "".to_string();
                    }
                    OutputFormat::Csv => {
                        rows = containers.par_iter().map(|p| p.to_csv()).collect();
                        separator = "\n".to_string();
                    }
                };
                println!("{}", rows.join(&separator));
            }
            OutputTarget::PostgreSQL => {
                let mut sql_error_rows = vec![];
                let mut sql_server_comment_rows = vec![];
                let mut sql_position_rows = vec![];
                let mut sql_status_rows = vec![];

                for container in containers {
                    let csv_row = container.to_csv();
                    match container.server_response {
                        ServerResponse::ParserError(_) => {
                            sql_error_rows.push(csv_row);
                        }
                        ServerResponse::ServerComment(_) => {
                            sql_server_comment_rows.push(csv_row);
                        }
                        ServerResponse::AprsPacket(packet) => match &packet.data {
                            AprsData::Position(_) => {
                                sql_position_rows.push(csv_row);
                            }
                            AprsData::Status(status) => {
                                sql_status_rows.push(csv_row);
                            }
                            _ => {}
                        },
                    }
                }

                self.insert_into_db(
                    "errors",
                    &ServerResponseContainer::csv_header_errors(),
                    sql_error_rows,
                );
                self.insert_into_db(
                    "server_comments",
                    &ServerResponseContainer::csv_header_server_comments(),
                    sql_server_comment_rows,
                );
                self.insert_into_db(
                    "positions",
                    &ServerResponseContainer::csv_header_positions(),
                    sql_position_rows,
                );
                self.insert_into_db(
                    "statuses",
                    &ServerResponseContainer::csv_header_statuses(),
                    sql_status_rows,
                );
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
                err,
                table_name,
                rows.join("\n")
            ),
        };
    }
}
