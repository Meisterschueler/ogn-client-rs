use ogn_parser::{AprsData, ServerResponse};

use crate::{
    containers::{
        comment_container::CommentContainer, parser_error_container::ParserErrorContainer,
        position_container::PositionContainer, server_comment_container::ServerCommentContainer,
        status_container::StatusContainer,
    },
    messages::server_response_container::ServerResponseContainer,
};

pub enum Container {
    Position(PositionContainer),
    Status(StatusContainer),
    ServerComment(ServerCommentContainer),
    ParserError(ParserErrorContainer),
    Comment(CommentContainer),
}

// from trait implementation for server response container to Container
impl From<ServerResponseContainer> for Container {
    fn from(server_response_container: ServerResponseContainer) -> Self {
        match server_response_container.server_response {
            ServerResponse::AprsPacket(packet) => match packet.data {
                AprsData::Position(position) => {
                    let mut container = PositionContainer {
                        ts: server_response_container.ts,
                        raw_message: server_response_container.raw_message,
                        receiver_ts: server_response_container.receiver_ts,
                        bearing: server_response_container.bearing,
                        distance: server_response_container.distance,
                        normalized_quality: server_response_container.normalized_signal_quality,
                        plausibility: server_response_container.plausibility,

                        src_call: packet.from,
                        dst_call: packet.to,
                        receiver: packet.via.last().cloned(),

                        receiver_time: position.timestamp,
                        messaging_supported: position.messaging_supported,
                        latitude: position.latitude,
                        longitude: position.longitude,
                        symbol_table: position.symbol_table,
                        symbol_code: position.symbol_code,

                        location: (*position.longitude, *position.latitude),

                        course: position.comment.course,
                        speed: position.comment.speed,
                        altitude: position.comment.altitude,
                        wind_direction: position.comment.wind_direction,
                        wind_speed: position.comment.wind_speed,
                        gust: position.comment.gust,
                        temperature: position.comment.temperature,
                        rainfall_1h: position.comment.rainfall_1h,
                        rainfall_24h: position.comment.rainfall_24h,
                        rainfall_midnight: position.comment.rainfall_midnight,
                        humidity: position.comment.humidity,
                        barometric_pressure: position.comment.barometric_pressure,
                        additional_precision: position.comment.additional_precision,
                        climb_rate: position.comment.climb_rate,
                        turn_rate: position.comment.turn_rate,
                        signal_quality: position.comment.signal_quality,
                        error: position.comment.error,
                        frequency_offset: position.comment.frequency_offset,
                        gps_quality: position.comment.gps_quality,
                        flight_level: position.comment.flight_level,
                        signal_power: position.comment.signal_power,
                        software_version: position.comment.software_version,
                        hardware_version: position.comment.hardware_version,
                        original_address: position.comment.original_address,
                        unparsed: position.comment.unparsed,

                        reserved: None,
                        address_type: None,
                        aircraft_type: None,
                        is_stealth: None,
                        is_notrack: None,
                        address: None,
                    };

                    if let Some(id) = position.comment.id {
                        container.reserved = id.reserved;
                        container.address_type = Some(id.address_type);
                        container.aircraft_type = Some(id.aircraft_type);
                        container.is_stealth = Some(id.is_stealth);
                        container.is_notrack = Some(id.is_notrack);
                        container.address = Some(id.address);
                    }

                    Container::Position(container)
                }
                AprsData::Status(status) => Container::Status(StatusContainer {
                    ts: server_response_container.ts,
                    raw_message: server_response_container.raw_message,
                    receiver_ts: server_response_container.receiver_ts,

                    src_call: packet.from,
                    dst_call: packet.to,
                    receiver: packet.via.last().cloned(),

                    receiver_time: status.timestamp,

                    version: status.comment.version,
                    platform: status.comment.platform,
                    cpu_load: status.comment.cpu_load,
                    ram_free: status.comment.ram_free,
                    ram_total: status.comment.ram_total,
                    ntp_offset: status.comment.ntp_offset,
                    ntp_correction: status.comment.ntp_correction,
                    voltage: status.comment.voltage,
                    amperage: status.comment.amperage,
                    cpu_temperature: status.comment.cpu_temperature,
                    visible_senders: status.comment.visible_senders,
                    latency: status.comment.latency,
                    senders: status.comment.senders,
                    rf_correction_manual: status.comment.rf_correction_manual,
                    rf_correction_automatic: status.comment.rf_correction_automatic,
                    noise: status.comment.noise,
                    senders_signal_quality: status.comment.senders_signal_quality,
                    senders_messages: status.comment.senders_messages,
                    good_senders_signal_quality: status.comment.good_senders_signal_quality,
                    good_senders: status.comment.good_senders,
                    good_and_bad_senders: status.comment.good_and_bad_senders,
                    unparsed: status.comment.unparsed,
                }),
                _ => Container::Comment(CommentContainer {}),
            },
            ServerResponse::ParserError(error) => Container::ParserError(ParserErrorContainer {
                ts: server_response_container.ts,
                raw_message: server_response_container.raw_message,
                error_message: error.to_string(),
            }),
            ServerResponse::ServerComment(server_comment) => {
                Container::ServerComment(ServerCommentContainer {
                    ts: server_response_container.ts,
                    receiver_ts: server_response_container.receiver_ts,

                    version: server_comment.version,
                    timestamp: server_comment.timestamp,
                    server: server_comment.server,
                    ip_address: server_comment.ip_address,
                    port: server_comment.port,
                })
            }
            _ => Container::Comment(CommentContainer {}),
        }
    }
}
