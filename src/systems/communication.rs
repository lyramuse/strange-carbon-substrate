// Communication System - Say and emote

use bevy::prelude::*;

use crate::domain::*;

pub fn communication_system(
    mut ev_reader: EventReader<CommunicationEvent>,
    query_players: Query<(&SubstrateIdentity, &Location)>,
    query_all_clients: Query<(&NetworkClient, &Location)>,
) {
    for event in ev_reader.read() {
        if let Ok((identity, sender_loc)) = query_players.get(event.sender) {
            let output = if event.is_emote {
                format!("\x1B[1;36m{} {}\x1B[0m", identity.name, event.message)
            } else {
                format!(
                    "\x1B[1;36m{} says, \"{}\"\x1B[0m",
                    identity.name, event.message
                )
            };

            // Broadcast to everyone in the same room
            for (client, client_loc) in query_all_clients.iter() {
                if client_loc.0 == sender_loc.0 {
                    let _ = client.tx.send(output.clone());
                }
            }
        }
    }
}
