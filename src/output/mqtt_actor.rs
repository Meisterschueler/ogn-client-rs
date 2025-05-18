use std::time::Duration;

use actix::prelude::*;
use rumqttc::{Client, MqttOptions};

use crate::{
    element_getter::ElementGetter, messages::server_response_container::ServerResponseContainer,
};

pub struct MqttActor {
    client: Client,
}

impl MqttActor {
    pub fn new(id: &str, host: &str, port: u16) -> Self {
        let mut options = MqttOptions::new(id, host, port);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, mut connection) = Client::new(options, 10);

        // Start background thread for Connection polling
        std::thread::spawn(move || {
            for _ in connection.iter() {
                // blockierend – ignorieren, weil wir nur senden
            }
        });

        MqttActor { client }
    }
}

impl Actor for MqttActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("MqttActor started");
    }
}

impl Handler<ServerResponseContainer> for MqttActor {
    type Result = ();

    fn handle(&mut self, msg: ServerResponseContainer, _: &mut Self::Context) {
        let elements = msg.get_elements();
        if let (Some(src_call), Some(distance), Some(altitude), Some(normalized_signal_quality)) = (
            elements.get("src_call"),
            elements.get("distance"),
            elements.get("altitude"),
            elements.get("normalized_signal_quality"),
        ) {
            let topic = format!("ogn/{src_call}");
            let payload = format!(
                "{{\"distance\": {distance}, \"altitude\": {altitude}, \"normalized_signal_quality\": {normalized_signal_quality}}}",
            );
            self.client
                .publish(topic, rumqttc::QoS::AtLeastOnce, false, payload)
                .unwrap();
        } else {
            error!(
                "Missing required elements in message: {:#?}",
                msg.server_response
            );
        }
    }
}
