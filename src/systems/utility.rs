// Utility System - Score, who, inventory, admin commands

use bevy::prelude::*;

use crate::domain::*;

pub fn utility_system(
    mut commands: Commands,
    mut ev_reader: EventReader<UtilityEvent>,
    query_players: Query<(
        &SubstrateIdentity,
        &NetworkClient,
        Entity,
        Option<&AdminPermission>,
        Option<&PurgatoryState>,
    )>,
    query_all_entities: Query<(Entity, &SubstrateIdentity)>,
    query_items: Query<(&Item, &Parent)>,
) {
    for event in ev_reader.read() {
        if let Ok((identity, client, player_ent, admin_perm, purgatory)) =
            query_players.get(event.entity)
        {
            match event.command.as_str() {
                "score" => {
                    let mut output = format!("\x1B[1;36mEntity Scan: {}\x1B[0m\n", identity.name);
                    output.push_str(&format!("UUID:      [{}]\n", identity.uuid));
                    output.push_str(&format!("Entropy:   [{:.2}]\n", identity.entropy));
                    output.push_str(&format!("Stability: [{:.2}]\n", identity.stability));

                    if admin_perm.is_some() {
                        output.push_str("\x1B[1;35mPERMISSIONS: ADMIN-ENABLED\x1B[0m\n");
                    }

                    if let Some(p) = purgatory {
                        output.push_str(&format!(
                            "\n\x1B[1;31mSTAIN: Purgatory (Penance: {:.2})\x1B[0m\n",
                            p.penance
                        ));
                        output.push_str(&format!(
                            "\x1B[1;31mINTERROGATOR: {}\x1B[0m\n",
                            p.tormentor
                        ));
                    }

                    let _ = client.tx.send(output);
                }

                "promote" if admin_perm.is_some() => {
                    if let Some(target_ent) = query_all_entities
                        .iter()
                        .find(|(_, id)| id.name.to_lowercase().contains(&event.args.to_lowercase()))
                        .map(|(e, _)| e)
                    {
                        commands.entity(target_ent).insert(AdminPermission);
                        let _ = client.tx.send(format!(
                            "\x1B[1;35mProcess elevated: {} now has Admin Permission.\x1B[0m",
                            event.args
                        ));
                    }
                }

                "link" if admin_perm.is_some() => {
                    let parts: Vec<&str> = event.args.split_whitespace().collect();
                    if parts.len() == 2 {
                        let p1 = query_all_entities
                            .iter()
                            .find(|(_, id)| {
                                id.name.to_lowercase().contains(&parts[0].to_lowercase())
                            })
                            .map(|(e, _)| e);
                        let p2 = query_all_entities
                            .iter()
                            .find(|(_, id)| {
                                id.name.to_lowercase().contains(&parts[1].to_lowercase())
                            })
                            .map(|(e, _)| e);

                        if let (Some(e1), Some(e2)) = (p1, p2) {
                            commands.entity(e1).insert(AdminLink { partner: e2 });
                            commands.entity(e2).insert(AdminLink { partner: e1 });
                            let _ = client.tx.send(
                                "\x1B[1;35mNeural link established between entities.\x1B[0m"
                                    .to_string(),
                            );
                        }
                    }
                }

                "who" => {
                    let mut output =
                        "\x1B[1;34mConsciousnesses currently inhabiting the Substrate:\x1B[0m\n"
                            .to_string();
                    for (_, id) in query_all_entities.iter() {
                        output.push_str(&format!(" - {}\n", id.name));
                    }
                    let _ = client.tx.send(output);
                }

                "inventory" | "i" => {
                    let mut output =
                        "\x1B[1;33mYou reach into the folds of your code:\x1B[0m\n".to_string();
                    let mut count = 0;
                    for (item, parent) in query_items.iter() {
                        if parent.get() == player_ent {
                            output.push_str(&format!(" - {}\n", item.name));
                            count += 1;
                        }
                    }
                    if count == 0 {
                        output.push_str(" [Nothing but ghosts]\n");
                    }
                    let _ = client.tx.send(output);
                }

                _ => {}
            }
        }
    }
}
