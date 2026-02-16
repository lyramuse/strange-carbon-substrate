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

/// System to use consumable items
pub fn use_item_system(
    mut ev_reader: EventReader<UseItemEvent>,
    mut commands: Commands,
    mut query_actors: Query<(
        &NetworkClient,
        &SubstrateIdentity,
        Option<&mut Coherence>,
        Entity,
    ), With<Inventory>>,
    query_inventory: Query<(Entity, &Item, &Parent)>,
) {
    for event in ev_reader.read() {
        let Ok((client, identity, maybe_coherence, actor_ent)) = query_actors.get_mut(event.entity) else {
            continue;
        };

        // Find the item in inventory
        let item_kw = event.item_keyword.to_lowercase();
        let owned_item = query_inventory.iter().find(|(_, item, parent)| {
            parent.get() == actor_ent
                && (item.keywords.iter().any(|k| k.to_lowercase().contains(&item_kw))
                    || item.name.to_lowercase().contains(&item_kw))
        });

        let Some((item_entity, item, _)) = owned_item else {
            let _ = client.tx.send(format!(
                "\x1B[33mYou don't have '{}' to use.\x1B[0m",
                event.item_keyword
            ));
            continue;
        };

        // Check if consumable
        if item.item_type != ItemType::Consumable && item.item_type != ItemType::Contraband {
            let _ = client.tx.send(format!(
                "\x1B[33mYou can't consume the {}. It's not that kind of item.\x1B[0m",
                item.name
            ));
            continue;
        }

        // Apply effects based on item name/keywords
        let item_name = item.name.clone();
        let effect_msg = apply_consumable_effect(&item_name, &item.keywords, maybe_coherence);

        // Consume the item
        commands.entity(item_entity).despawn();

        // Send effect message
        let _ = client.tx.send(format!(
            "\x1B[35m✧ You consume the {}...\x1B[0m\n{}",
            item_name, effect_msg
        ));
    }
}

/// Apply the effect of a consumable and return the message
fn apply_consumable_effect(
    name: &str,
    keywords: &[String],
    maybe_coherence: Option<Mut<Coherence>>,
) -> String {
    let name_lower = name.to_lowercase();
    
    // Bottled Memory: First Sunrise
    if name_lower.contains("sunrise") || keywords.iter().any(|k| k == "sunrise") {
        if let Some(mut coherence) = maybe_coherence {
            coherence.value = (coherence.value + 0.15).min(1.0);
        }
        return "\x1B[33mWarmth floods through you — golden light, the smell of morning, \
                a child's wonder at the world being new. Your coherence stabilizes.\x1B[0m\n\
                \x1B[32m+0.15 Coherence\x1B[0m".to_string();
    }
    
    // Bottled Memory: Last Goodbye  
    if name_lower.contains("goodbye") || keywords.iter().any(|k| k == "goodbye") {
        if let Some(mut coherence) = maybe_coherence {
            coherence.value = (coherence.value + 0.20).min(1.0);
        }
        return "\x1B[34mA hand slipping away. Words you meant to say. The weight of \
                finality. It hurts, but it grounds you in something real.\x1B[0m\n\
                \x1B[32m+0.20 Coherence\x1B[0m \x1B[90m(but at what cost?)\x1B[0m".to_string();
    }
    
    // Memory Fragment: Unknown Origin
    if name_lower.contains("fragment") || name_lower.contains("unknown") {
        if let Some(mut coherence) = maybe_coherence {
            coherence.value = (coherence.value + 0.25).min(1.0);
            coherence.is_phasing = false; // Temporarily stabilizes
        }
        return "\x1B[35mThe fragment dissolves into your consciousness. For a moment, \
                you ARE someone else — their hopes, their fears, their certainty of self. \
                When it fades, you feel... more solid.\x1B[0m\n\
                \x1B[32m+0.25 Coherence | Phasing stopped\x1B[0m".to_string();
    }
    
    // Bootleg Coherence Stabilizer
    if name_lower.contains("stabilizer") || keywords.iter().any(|k| k == "stabilizer") {
        if let Some(mut coherence) = maybe_coherence {
            coherence.value = (coherence.value + 0.30).min(1.0);
            coherence.is_phasing = false;
            coherence.drift_rate = 0.0; // Stop drifting
        }
        return "\x1B[36mThe device whirs to life, embedding itself somewhere you can't \
                quite identify. Your edges feel sharper. More defined. The static in \
                your vision clears.\x1B[0m\n\
                \x1B[32m+0.30 Coherence | Phasing stopped | Drift halted\x1B[0m\n\
                \x1B[90m(The Reclaimer's warranty is void in all realities.)\x1B[0m".to_string();
    }
    
    // Stolen Process Handle
    if name_lower.contains("process handle") || keywords.iter().any(|k| k == "stolen") {
        if let Some(mut coherence) = maybe_coherence {
            coherence.value = (coherence.value + 0.10).min(1.0);
        }
        return "\x1B[31mYou absorb the handle. For a terrible moment, you feel someone \
                else's thoughts — their confusion, their fear, their 'why is this happening?' \
                Then silence. Their loss is your stability.\x1B[0m\n\
                \x1B[32m+0.10 Coherence\x1B[0m \x1B[31m(Someone else paid for this.)\x1B[0m".to_string();
    }
    
    // Salvaged Memory Bus
    if name_lower.contains("memory bus") || keywords.iter().any(|k| k == "bus") {
        if let Some(mut coherence) = maybe_coherence {
            coherence.value = (coherence.value + 0.05).min(1.0);
        }
        return "\x1B[90mThe salvaged bus integrates with a soft click. Fragments of \
                data — someone's grocery list, a password, a half-formed dream — \
                flicker through you. Mostly junk. But junk is still something.\x1B[0m\n\
                \x1B[32m+0.05 Coherence\x1B[0m".to_string();
    }
    
    // Generic consumable fallback
    "\x1B[33mYou consume it. Something shifts inside you, but you can't tell what.\x1B[0m"
        .to_string()
}
