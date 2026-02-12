// Stream System - Handle velocity/pressure in high-speed network zones
//
// The Packet Stream pushes back. Linger too long and you'll be ejected
// back toward safety. Keep moving or get swept away.
//
// TODO(@lyra): Add "swimming upstream" mechanic - high Entropy lets you resist longer

use bevy::prelude::*;

use crate::domain::*;

/// System to apply stream pressure and handle push-backs
pub fn stream_pressure_system(
    time: Res<Time>,
    mut query_entities: Query<(
        Entity,
        &Location,
        &mut StreamPressure,
        &SubstrateIdentity,
        Option<&NetworkClient>,
    )>,
    query_zones: Query<(&StreamZone, Option<&Room>)>,
    mut move_events: EventWriter<MoveEvent>,
) {
    for (entity, location, mut pressure, identity, maybe_client) in query_entities.iter_mut() {
        // Check if current room is a StreamZone
        if let Ok((zone, maybe_room)) = query_zones.get(location.0) {
            // Apply pressure
            pressure.current += zone.pressure_rate * time.delta_seconds();

            // Warn at 50% and 75%
            if let Some(client) = maybe_client {
                let prev = pressure.current - (zone.pressure_rate * time.delta_seconds());
                
                if prev < 0.5 && pressure.current >= 0.5 {
                    let _ = client.tx.send(
                        "\x1B[33mThe stream pressure intensifies. You feel yourself being pushed back.\x1B[0m"
                            .to_string(),
                    );
                }
                if prev < 0.75 && pressure.current >= 0.75 {
                    let _ = client.tx.send(
                        "\x1B[31mWARNING: Stream pressure critical! Move deeper or retreat!\x1B[0m"
                            .to_string(),
                    );
                }
            }

            // Check for push-back
            if pressure.current >= pressure.threshold {
                if let Some(client) = maybe_client {
                    let room_name = maybe_room.map(|r| r.title.as_str()).unwrap_or("the stream");
                    let _ = client.tx.send(format!(
                        "\x1B[1;31mThe stream overcomes you! You're swept back from {}!\x1B[0m",
                        room_name
                    ));
                }

                // Reset pressure and trigger movement
                pressure.current = 0.3; // Don't reset to 0 - still in danger zone
                
                // Push back west (toward safety)
                move_events.send(MoveEvent {
                    entity,
                    direction: "west".to_string(),
                });
            }
        } else {
            // Not in a StreamZone - decay pressure
            if pressure.current > 0.0 {
                pressure.current = (pressure.current - pressure.decay_rate * time.delta_seconds())
                    .max(0.0);
                
                // Notify when pressure fully dissipates
                if let Some(client) = maybe_client {
                    if pressure.current == 0.0 {
                        let _ = client.tx.send(
                            "\x1B[32mThe stream pressure fades. You've reached stable ground.\x1B[0m"
                                .to_string(),
                        );
                    }
                }
            }
        }
    }
}

/// Give new players a StreamPressure component when they connect
/// (This should be called when spawning player entities)
pub fn init_stream_pressure() -> StreamPressure {
    StreamPressure {
        current: 0.0,
        threshold: 1.0,
        decay_rate: 0.15,
    }
}
