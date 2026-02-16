// Movement System - Handle directional movement

use bevy::prelude::*;

use crate::domain::*;

pub fn move_system(
    mut ev_reader: EventReader<MoveEvent>,
    mut query_players: Query<(&mut Location, &NetworkClient)>,
    query_rooms: Query<&Exits>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let Ok((mut location, client)) = query_players.get_mut(event.entity) {
            if let Ok(exits) = query_rooms.get(location.0) {
                if let Some(target_room) = exits.get(&event.direction) {
                    location.0 = target_room;
                    look_writer.send(LookEvent {
                        entity: event.entity,
                        target: None,
                    });
                } else {
                    let _ = client.tx.send(
                        "\x1B[31mThe path is barred by twisted wrought iron and static.\x1B[0m"
                            .to_string(),
                    );
                }
            }
        }
    }
}
