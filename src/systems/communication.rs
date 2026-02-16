// Communication System - Say and emote
//
// NPCs can respond to player speech based on keyword triggers.
// "The Substrate listens. Sometimes it answers back."

use bevy::prelude::*;

use crate::domain::*;

pub fn communication_system(
    mut ev_reader: EventReader<CommunicationEvent>,
    query_players: Query<(&SubstrateIdentity, &Location)>,
    query_all_clients: Query<(&NetworkClient, &Location)>,
    query_npcs: Query<(&SubstrateIdentity, &Location, &Dialogue), With<NonPlayer>>,
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

            // Check for NPC responses (only for say, not emote)
            if !event.is_emote {
                let message_lower = event.message.to_lowercase();
                
                for (npc_id, npc_loc, dialogue) in query_npcs.iter() {
                    // NPC must be in the same room
                    if npc_loc.0 != sender_loc.0 {
                        continue;
                    }

                    // Check for keyword matches
                    let mut response: Option<&str> = None;
                    
                    for entry in &dialogue.responses {
                        if entry.keywords.iter().any(|k| message_lower.contains(k)) {
                            response = Some(&entry.response);
                            break;
                        }
                    }

                    // If no specific response, use default (with some randomness)
                    let npc_response = response.unwrap_or_else(|| {
                        // Only respond ~30% of the time to unrecognized speech
                        if rand::random::<f32>() < 0.3 {
                            &dialogue.default_response
                        } else {
                            return;
                        }
                    });

                    // Format and send NPC response
                    let npc_output = format!(
                        "\n\x1B[35m{} says, \"{}\"\x1B[0m",
                        npc_id.name, npc_response
                    );

                    // Send to all in room
                    for (client, client_loc) in query_all_clients.iter() {
                        if client_loc.0 == sender_loc.0 {
                            let _ = client.tx.send(npc_output.clone());
                        }
                    }
                }
            }
        }
    }
}
