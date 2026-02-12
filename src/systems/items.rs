// Item System - Get, drop, and interact with items

use bevy::prelude::*;

use crate::domain::*;

pub fn item_action_system(
    mut ev_reader: EventReader<ActionEvent>,
    mut commands: Commands,
    query_actors: Query<(&Location, &NetworkClient, Entity), With<Inventory>>,
    query_items: Query<(Entity, &Item, &Location)>,
    query_inventory: Query<(Entity, &Item, &Parent)>,
) {
    for event in ev_reader.read() {
        if let Ok((location, client, actor_ent)) = query_actors.get(event.entity) {
            match event.action.as_str() {
                "get" | "take" => {
                    let mut found = false;
                    for (item_ent, item, item_loc) in query_items.iter() {
                        if item_loc.0 == location.0
                            && item.keywords.contains(&event.target.to_lowercase())
                        {
                            commands
                                .entity(item_ent)
                                .remove::<Location>()
                                .set_parent(actor_ent);
                            let _ = client.tx.send(format!(
                                "\x1B[33mYou interface with the {} and pull it into your local cache.\x1B[0m",
                                item.name
                            ));
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        let _ = client.tx.send(
                            "\x1B[31mThe shadows hide no such object.\x1B[0m".to_string(),
                        );
                    }
                }

                "drop" => {
                    let mut found = false;
                    for (item_ent, item, parent) in query_inventory.iter() {
                        if parent.get() == actor_ent
                            && item.keywords.contains(&event.target.to_lowercase())
                        {
                            commands
                                .entity(item_ent)
                                .remove_parent()
                                .insert(Location(location.0));
                            let _ = client.tx.send(format!(
                                "\x1B[33mYou de-allocate the {} and drop it into the environment.\x1B[0m",
                                item.name
                            ));
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        let _ = client.tx.send(
                            "\x1B[31mYou aren't carrying that process.\x1B[0m".to_string(),
                        );
                    }
                }

                _ => {}
            }
        }
    }
}
