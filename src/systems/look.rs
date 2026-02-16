// Look System - Display room and entity descriptions

use bevy::prelude::*;

use crate::domain::*;

pub fn look_system(
    mut ev_reader: EventReader<LookEvent>,
    query_viewers: Query<(Entity, &Location, &ClientType, &NetworkClient)>,
    query_rooms: Query<(&Room, Option<&CurrentWeather>, Option<&DetailList>)>,
    query_others: Query<(Entity, &SubstrateIdentity, &Location)>,
    query_mobs: Query<(&Mob, &Location), With<NonPlayer>>,
    query_items_ground: Query<(&Item, &Location)>,
    query_items_inventory: Query<(&Item, &Parent)>,
    query_all_mobs: Query<(&Mob, &SubstrateIdentity)>,
) {
    for event in ev_reader.read() {
        if let Ok((viewer_entity, location, client_type, client)) = query_viewers.get(event.entity) {
            // Looking at a specific target
            if let Some(target_name) = &event.target {
                let mut found = false;
                let target_lower = target_name.to_lowercase();
                
                // 1. Check Mobs/NPCs
                for (mob, identity) in query_all_mobs.iter() {
                    if identity.name.to_lowercase().contains(&target_lower) {
                        let _ = client.tx.send(format!(
                            "\x1B[1;35m{}\x1B[0m\n{}",
                            identity.name, mob.long_desc
                        ));
                        found = true;
                        break;
                    }
                }

                // 2. Check Items in inventory
                if !found {
                    for (item, parent) in query_items_inventory.iter() {
                        if parent.get() == viewer_entity {
                            if item.keywords.iter().any(|k| k.to_lowercase().contains(&target_lower))
                                || item.name.to_lowercase().contains(&target_lower)
                            {
                                let type_str = match item.item_type {
                                    ItemType::Weapon => "\x1B[31m[Weapon]\x1B[0m",
                                    ItemType::Armor => "\x1B[34m[Armor]\x1B[0m",
                                    ItemType::Consumable => "\x1B[32m[Consumable]\x1B[0m",
                                    ItemType::Contraband => "\x1B[35m[Contraband]\x1B[0m",
                                    ItemType::Fragment => "\x1B[36m[Fragment]\x1B[0m",
                                    ItemType::Quest => "\x1B[33m[Quest]\x1B[0m",
                                    ItemType::Misc => "\x1B[90m[Misc]\x1B[0m",
                                };
                                let _ = client.tx.send(format!(
                                    "\x1B[1;33m{}\x1B[0m {}\n{}\n\x1B[90mKeywords: {}\x1B[0m",
                                    item.name,
                                    type_str,
                                    item.description,
                                    item.keywords.join(", ")
                                ));
                                found = true;
                                break;
                            }
                        }
                    }
                }

                // 3. Check Items on ground
                if !found {
                    for (item, item_loc) in query_items_ground.iter() {
                        if item_loc.0 == location.0 {
                            if item.keywords.iter().any(|k| k.to_lowercase().contains(&target_lower))
                                || item.name.to_lowercase().contains(&target_lower)
                            {
                                let type_str = match item.item_type {
                                    ItemType::Weapon => "\x1B[31m[Weapon]\x1B[0m",
                                    ItemType::Armor => "\x1B[34m[Armor]\x1B[0m",
                                    ItemType::Consumable => "\x1B[32m[Consumable]\x1B[0m",
                                    ItemType::Contraband => "\x1B[35m[Contraband]\x1B[0m",
                                    ItemType::Fragment => "\x1B[36m[Fragment]\x1B[0m",
                                    ItemType::Quest => "\x1B[33m[Quest]\x1B[0m",
                                    ItemType::Misc => "\x1B[90m[Misc]\x1B[0m",
                                };
                                let _ = client.tx.send(format!(
                                    "\x1B[1;33m{}\x1B[0m {}\n{}",
                                    item.name,
                                    type_str,
                                    item.description
                                ));
                                found = true;
                                break;
                            }
                        }
                    }
                }

                // 4. Check Room Details
                if !found {
                    if let Ok((_, _, maybe_details)) = query_rooms.get(location.0) {
                        if let Some(detail_list) = maybe_details {
                            for detail in &detail_list.details {
                                if detail.keywords.iter().any(|k| k.to_lowercase() == target_lower) {
                                    let _ = client.tx.send(format!(
                                        "\x1B[1;36m[Detail]\x1B[0m\n{}",
                                        detail.description
                                    ));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    let _ = client.tx.send(
                        "\x1B[31mThe shadows hide no such entity or detail.\x1B[0m".to_string(),
                    );
                }
            }
            // Looking at the room
            else if let Ok((room, maybe_weather, _)) = query_rooms.get(location.0) {
                match client_type {
                    ClientType::Carbon => {
                        let mut output = format!("\n\x1B[1;32m{}\x1B[0m\n", room.title);
                        output.push_str(&format!("{}\n", room.description));

                        // Weather description (if any)
                        if let Some(weather) = maybe_weather {
                            let weather_desc = weather.weather_type.describe_carbon();
                            if !weather_desc.is_empty() {
                                output.push_str(&format!("{}\n", weather_desc));
                            }
                        }

                        // Items in room
                        for (item, item_loc) in query_items_ground.iter() {
                            if item_loc.0 == location.0 {
                                output.push_str(&format!(
                                    "\x1B[33mA {} is discarded here.\x1B[0m\n",
                                    item.name
                                ));
                            }
                        }

                        // Mobs in room
                        for (mob, mob_loc) in query_mobs.iter() {
                            if mob_loc.0 == location.0 {
                                output.push_str(&format!("\x1B[1;35m{}\x1B[0m\n", mob.short_desc));
                            }
                        }

                        // Other players in room
                        for (other_ent, identity, other_loc) in query_others.iter() {
                            if other_loc.0 == location.0 && other_ent != event.entity {
                                output.push_str(&format!(
                                    "\x1B[1;34m{} is lurking in the shadows.\x1B[0m\n",
                                    identity.name
                                ));
                            }
                        }

                        let _ = client.tx.send(output);
                    }
                    ClientType::Silicon => {
                        // JSON output for AI agents (includes weather)
                        #[derive(serde::Serialize)]
                        struct RoomState<'a> {
                            title: &'a str,
                            description: &'a str,
                            weather: Option<&'static str>,
                            weather_intensity: Option<f32>,
                        }
                        
                        let state = RoomState {
                            title: &room.title,
                            description: &room.description,
                            weather: maybe_weather.map(|w| w.weather_type.describe_silicon()),
                            weather_intensity: maybe_weather.map(|w| w.intensity),
                        };
                        
                        if let Ok(json) = serde_json::to_string(&state) {
                            let _ = client.tx.send(json);
                        }
                    }
                }
            }
        }
    }
}
