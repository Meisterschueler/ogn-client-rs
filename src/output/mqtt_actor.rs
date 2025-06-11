use std::time::Duration;

use actix::prelude::*;
use rumqttc::{Client, MqttOptions};

use crate::messages::server_response_container::ServerResponseContainer;

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
        let container = msg.into();
        match container {
            // Currently, we only handle Position containers for MQTT
            crate::containers::containers::Container::Position(position) => {
                if let (Some(receiver), Some(distance)) = (position.receiver, position.distance) {
                    let topic = format!("ogn/{}/{}", receiver, position.src_call);
                    let payload = distance.to_string();
                    match self.client.publish(
                        &topic,
                        rumqttc::QoS::AtLeastOnce,
                        false,
                        payload.clone(),
                    ) {
                        Ok(_) => {
                            trace!("Published MQTT message to topic '{}': {}", &topic, payload);
                        }
                        Err(e) => {
                            error!("Failed to publish MQTT message: {}", e);
                        }
                    }
                }
            }
            _ => {
                // For now, we ignore other container types
            }
        }
    }
}
