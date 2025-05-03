use std::{io::Write, time::Duration};

use actix::prelude::*;
use ogn_parser::{AprsData, ServerResponse};
use postgres::{Client, NoTls};

use crate::messages::server_response_container::ServerResponseContainer;

pub struct PostgreSQLActor {
    pub client: Option<postgres::Client>,

    pub containers: Vec<ServerResponseContainer>,
}

impl PostgreSQLActor {
    pub fn new(database_url: &str) -> Self {
        PostgreSQLActor {
            client: Client::connect(database_url, NoTls).ok(),
            containers: Vec::new(),
        }
    }

    fn flush(&mut self) {
        let mut sql_error_rows = vec![];
        let mut sql_server_comment_rows = vec![];
        let mut sql_position_rows = vec![];
        let mut sql_status_rows = vec![];

        for container in &self.containers {
            let csv_row = container.to_csv();
            match &container.server_response {
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
                    AprsData::Status(_) => {
                        sql_status_rows.push(csv_row);
                    }
                    _ => {}
                },
            }
        }

        self.insert_into_db(
            "errors",
            &ServerResponseContainer::csv_header_errors(),
            &sql_error_rows,
        );
        self.insert_into_db(
            "server_comments",
            &ServerResponseContainer::csv_header_server_comments(),
            &sql_server_comment_rows,
        );
        self.insert_into_db(
            "positions",
            &ServerResponseContainer::csv_header_positions(),
            &sql_position_rows,
        );
        self.insert_into_db(
            "statuses",
            &ServerResponseContainer::csv_header_statuses(),
            &sql_status_rows,
        );

        self.containers.clear();
    }

    fn insert_into_db(&mut self, table_name: &str, csv_header: &str, rows: &[String]) {
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

impl Actor for PostgreSQLActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("PostgreSQLActor started");
        ctx.run_interval(Duration::from_secs(1), |act, _ctx| {
            act.flush();
        });
    }
}

impl Handler<ServerResponseContainer> for PostgreSQLActor {
    type Result = ();

    fn handle(&mut self, msg: ServerResponseContainer, _: &mut Self::Context) {
        self.containers.push(msg);
    }
}
