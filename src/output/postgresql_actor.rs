use std::{io::Write, time::Duration};

use actix::prelude::*;
use csv::WriterBuilder;
use postgres::{Client, NoTls};

use crate::{
    containers::{
        containers::Container, parser_error_container::ParserErrorContainer,
        position_container::PositionContainer, server_comment_container::ServerCommentContainer,
        status_container::StatusContainer,
    },
    messages::server_response_container::ServerResponseContainer,
};

pub struct PostgreSQLActor {
    pub client: Option<postgres::Client>,

    pub position_containers: Vec<PositionContainer>,
    pub status_containers: Vec<StatusContainer>,
    pub server_comment_containers: Vec<ServerCommentContainer>,
    pub parser_error_containers: Vec<ParserErrorContainer>,
}

impl PostgreSQLActor {
    pub fn new(database_url: &str) -> Self {
        PostgreSQLActor {
            client: Client::connect(database_url, NoTls).ok(),

            position_containers: vec![],
            status_containers: vec![],
            server_comment_containers: vec![],
            parser_error_containers: vec![],
        }
    }

    fn get_header_and_body(&self, containers: &[impl serde::Serialize]) -> (String, Vec<u8>) {
        let mut buffer = Vec::new();
        {
            let mut writer = WriterBuilder::new()
                .has_headers(true)
                .from_writer(&mut buffer);

            for container in containers {
                writer.serialize(container).unwrap();
            }
            writer.flush().unwrap();
        }

        let content = String::from_utf8(buffer).unwrap();
        let header = content.lines().next().unwrap().to_string();
        let body = content
            .lines()
            .skip(1)
            .collect::<Vec<&str>>()
            .join("\n")
            .into_bytes();

        (header, body)
    }

    fn flush(&mut self) {
        if !self.position_containers.is_empty() {
            let (header, body) = self.get_header_and_body(&self.position_containers);
            self.insert_into_db("positions", &header, &body);
            self.position_containers.clear();
        }

        if !self.status_containers.is_empty() {
            let (header, body) = self.get_header_and_body(&self.status_containers);
            self.insert_into_db("statuses", &header, &body);
            self.status_containers.clear();
        }

        if !self.server_comment_containers.is_empty() {
            let (header, body) = self.get_header_and_body(&self.server_comment_containers);
            self.insert_into_db("server_comments", &header, &body);
            self.server_comment_containers.clear();
        }

        if !self.parser_error_containers.is_empty() {
            let (header, body) = self.get_header_and_body(&self.parser_error_containers);
            self.insert_into_db("errors", &header, &body);
            self.parser_error_containers.clear();
        }
    }

    fn insert_into_db(&mut self, table_name: &str, header: &str, body: &[u8]) {
        let client = self.client.as_mut().unwrap();
        let sql_header = format!("COPY {table_name} ({header}) FROM STDIN WITH (FORMAT CSV)");
        let mut copy_stdin = client.copy_in(&sql_header).unwrap();
        copy_stdin.write_all(body).unwrap();
        match copy_stdin.finish() {
            Ok(_) => trace!(
                "{} messages inserted into table '{}'",
                body.len(),
                table_name
            ),
            Err(err) => error!("Error: {err}\nTable: {table_name}\nRows: {body:#?}"),
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
        match msg.into() {
            Container::Position(position) => {
                self.position_containers.push(position);
            }
            Container::Status(status) => {
                self.status_containers.push(status);
            }
            Container::ServerComment(server_comment) => {
                self.server_comment_containers.push(server_comment);
            }
            Container::ParserError(parser_error) => {
                self.parser_error_containers.push(parser_error);
            }
            Container::Comment(comment_container) => todo!(),
        }
    }
}
