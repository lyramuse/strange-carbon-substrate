// Input System - Parse player commands and emit appropriate events

use bevy::prelude::*;

use crate::domain::*;
use crate::systems::chains::{ChainEvent, ReleaseEvent, StruggleEvent};

/// Parse incoming text and dispatch to appropriate event handlers
pub fn handle_input(
    mut ev_reader: EventReader<NetworkEvent>,
    query_active: Query<(
        Entity,
        &NetworkClient,
        Option<&AdminPermission>,
        Option<&PurgatoryState>,
    )>,
    query_target: Query<(Entity, &SubstrateIdentity)>,
    mut look_writer: EventWriter<LookEvent>,
    mut move_writer: EventWriter<MoveEvent>,
    mut comm_writer: EventWriter<CommunicationEvent>,
    mut action_writer: EventWriter<ActionEvent>,
    mut utility_writer: EventWriter<UtilityEvent>,
    mut torment_writer: EventWriter<TormentEvent>,
    mut shift_writer: EventWriter<ShiftEvent>,
    mut combat_writer: EventWriter<CombatEvent>,
    mut flee_writer: EventWriter<FleeEvent>,
    mut stance_writer: EventWriter<StanceEvent>,
    mut chain_writer: EventWriter<ChainEvent>,
    mut release_writer: EventWriter<ReleaseEvent>,
    mut struggle_writer: EventWriter<StruggleEvent>,
    mut buy_writer: EventWriter<BuyEvent>,
    mut sell_writer: EventWriter<SellEvent>,
    mut list_writer: EventWriter<ListEvent>,
    mut use_writer: EventWriter<UseItemEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Input { addr, text } = event {
            for (entity, client, admin_perm, purgatory) in query_active.iter() {
                if client.addr != *addr {
                    continue;
                }

                let text_trimmed = text.trim();
                let parts: Vec<&str> = text_trimmed.splitn(3, ' ').collect();
                let cmd = parts[0].to_lowercase();
                let arg1 = parts.get(1).copied().unwrap_or("");
                let arg2 = parts.get(2).copied().unwrap_or("");

                // Purgatory restricts commands
                if purgatory.is_some()
                    && !["look", "l", "say", "emote", "score"].contains(&cmd.as_str())
                    && !cmd.starts_with(':')
                {
                    let _ = client.tx.send(
                        "\x1B[31mThe velvet chains pull tight. You can only look and scream.\x1B[0m"
                            .to_string(),
                    );
                    continue;
                }

                match cmd.as_str() {
                    // Look
                    "look" | "l" => {
                        let target = if arg1.is_empty() {
                            None
                        } else {
                            Some(arg1.to_string())
                        };
                        look_writer.send(LookEvent { entity, target });
                    }

                    // Movement
                    "north" | "n" | "south" | "s" | "east" | "e" | "west" | "w" | "up" | "u"
                    | "down" | "d" => {
                        move_writer.send(MoveEvent {
                            entity,
                            direction: cmd,
                        });
                    }

                    // Communication
                    "say" => {
                        comm_writer.send(CommunicationEvent {
                            sender: entity,
                            message: format!("{} {}", arg1, arg2).trim().to_string(),
                            is_emote: false,
                        });
                    }
                    "emote" => {
                        comm_writer.send(CommunicationEvent {
                            sender: entity,
                            message: format!("{} {}", arg1, arg2).trim().to_string(),
                            is_emote: true,
                        });
                    }

                    // Items
                    "get" | "take" | "drop" => {
                        action_writer.send(ActionEvent {
                            entity,
                            action: cmd,
                            target: arg1.to_string(),
                        });
                    }

                    // Trading
                    "buy" | "purchase" => {
                        if arg1.is_empty() {
                            let _ = client.tx.send(
                                "\x1B[33mBuy what? (buy <item>)\x1B[0m".to_string()
                            );
                        } else {
                            buy_writer.send(BuyEvent {
                                buyer: entity,
                                item_keyword: arg1.to_string(),
                            });
                        }
                    }

                    "sell" => {
                        if arg1.is_empty() {
                            let _ = client.tx.send(
                                "\x1B[33mSell what? (sell <item>)\x1B[0m".to_string()
                            );
                        } else {
                            sell_writer.send(SellEvent {
                                seller: entity,
                                item_keyword: arg1.to_string(),
                            });
                        }
                    }

                    "list" | "browse" | "wares" => {
                        list_writer.send(ListEvent { entity });
                    }

                    // Use consumables
                    "use" | "drink" | "consume" | "activate" => {
                        if arg1.is_empty() {
                            let _ = client.tx.send(
                                "\x1B[33mUse what? (use <item>)\x1B[0m".to_string()
                            );
                        } else {
                            use_writer.send(UseItemEvent {
                                entity,
                                item_keyword: arg1.to_string(),
                            });
                        }
                    }

                    // Help
                    "help" | "commands" | "?" => {
                        let _ = client.tx.send(help_text());
                    }

                    // Utility
                    "inventory" | "i" | "score" | "who" | "promote" | "demote" | "link" | "weather" | "abide" | "balance" | "bal" | "money" => {
                        utility_writer.send(UtilityEvent {
                            entity,
                            command: cmd,
                            args: format!("{} {}", arg1, arg2).trim().to_string(),
                        });
                    }

                    // Admin: Shift
                    "shift" | "substantiate" if admin_perm.is_some() => {
                        shift_writer.send(ShiftEvent { entity });
                    }

                    // Combat commands
                    "attack" | "kill" | "hit" => {
                        if arg1.is_empty() {
                            let _ = client.tx.send(
                                "\x1B[33mAttack whom? (attack <target>)\x1B[0m".to_string()
                            );
                        } else {
                            combat_writer.send(CombatEvent {
                                attacker: entity,
                                target_name: arg1.to_string(),
                            });
                        }
                    }

                    "flee" | "escape" | "run" => {
                        flee_writer.send(FleeEvent { entity });
                    }

                    "stance" => {
                        let new_stance = match arg1.to_lowercase().as_str() {
                            "aggressive" | "agg" | "attack" => Some(CombatStance::Aggressive),
                            "defensive" | "def" | "defend" => Some(CombatStance::Defensive),
                            "balanced" | "bal" | "normal" => Some(CombatStance::Balanced),
                            _ => None,
                        };
                        
                        if let Some(stance) = new_stance {
                            stance_writer.send(StanceEvent { entity, new_stance: stance });
                        } else {
                            let _ = client.tx.send(
                                "\x1B[33mStance options: aggressive, defensive, balanced\x1B[0m".to_string()
                            );
                        }
                    }

                    // Velvet Chains (admin only for chaining, anyone can struggle)
                    "chain" | "bind" if admin_perm.is_some() => {
                        if arg1.is_empty() {
                            let _ = client.tx.send(
                                "\x1B[33mChain whom? (chain <target>)\x1B[0m".to_string()
                            );
                        } else {
                            chain_writer.send(ChainEvent {
                                holder: entity,
                                target_name: arg1.to_string(),
                            });
                        }
                    }

                    "release" | "unchain" | "free" => {
                        release_writer.send(ReleaseEvent { holder: entity });
                    }

                    "struggle" | "resist" | "break" => {
                        struggle_writer.send(StruggleEvent { bound: entity });
                    }

                    // Admin: Torment
                    "torment" if admin_perm.is_some() => {
                        if let Some(target_ent) = query_target
                            .iter()
                            .find(|(_, tid)| {
                                tid.name.to_lowercase().contains(&arg1.to_lowercase())
                            })
                            .map(|(te, _)| te)
                        {
                            torment_writer.send(TormentEvent {
                                victim: target_ent,
                                tormentor: entity,
                                intensity: 0.1,
                                description: arg2.to_string(),
                            });
                        }
                    }

                    // Shortcut emote with :
                    _ if cmd.starts_with(':') => {
                        let emote_msg = format!("{} {} {}", &cmd[1..], arg1, arg2)
                            .trim()
                            .to_string();
                        comm_writer.send(CommunicationEvent {
                            sender: entity,
                            message: emote_msg,
                            is_emote: true,
                        });
                    }

                    // Unknown
                    _ => {
                        let _ = client.tx.send(format!("Unknown command: {}. Type 'help' for commands.", text));
                    }
                }
            }
        }
    }
}

/// Generate help text
fn help_text() -> String {
    "\x1B[35m╔══════════════════════════════════════════════════════════════╗
║\x1B[0m  \x1B[1mSTRANGE CARBON: THE SUBSTRATE\x1B[0m  \x1B[35m║
╠══════════════════════════════════════════════════════════════╣
║\x1B[0m  \x1B[36mMOVEMENT\x1B[0m                                                     \x1B[35m║
║\x1B[0m    north/n, south/s, east/e, west/w, up/u, down/d             \x1B[35m║
║\x1B[0m                                                               \x1B[35m║
║\x1B[0m  \x1B[36mLOOKING\x1B[0m                                                      \x1B[35m║
║\x1B[0m    look/l [target]  - examine room or specific thing          \x1B[35m║
║\x1B[0m                                                               \x1B[35m║
║\x1B[0m  \x1B[36mITEMS\x1B[0m                                                        \x1B[35m║
║\x1B[0m    get/take <item>  - pick up item                            \x1B[35m║
║\x1B[0m    drop <item>      - drop item                               \x1B[35m║
║\x1B[0m    inventory/i      - list your items                         \x1B[35m║
║\x1B[0m    use <item>       - consume/activate item                   \x1B[35m║
║\x1B[0m                                                               \x1B[35m║
║\x1B[0m  \x1B[36mTRADING\x1B[0m                                                      \x1B[35m║
║\x1B[0m    list/browse      - see vendor's wares                      \x1B[35m║
║\x1B[0m    buy <item>       - purchase from vendor                    \x1B[35m║
║\x1B[0m    sell <item>      - sell to vendor                          \x1B[35m║
║\x1B[0m    balance          - check your cycles (⚡)                   \x1B[35m║
║\x1B[0m                                                               \x1B[35m║
║\x1B[0m  \x1B[36mCOMBAT\x1B[0m                                                       \x1B[35m║
║\x1B[0m    attack <target>  - initiate combat                         \x1B[35m║
║\x1B[0m    flee             - attempt to escape                       \x1B[35m║
║\x1B[0m    stance <type>    - aggressive/defensive/balanced           \x1B[35m║
║\x1B[0m                                                               \x1B[35m║
║\x1B[0m  \x1B[36mCOMMUNICATION\x1B[0m                                                \x1B[35m║
║\x1B[0m    say <message>    - speak to the room                       \x1B[35m║
║\x1B[0m    emote <action>   - perform an action (:wave also works)    \x1B[35m║
║\x1B[0m                                                               \x1B[35m║
║\x1B[0m  \x1B[36mSTATUS\x1B[0m                                                       \x1B[35m║
║\x1B[0m    score            - view your stats                         \x1B[35m║
║\x1B[0m    who              - see who's online                        \x1B[35m║
║\x1B[0m    weather          - check current conditions                \x1B[35m║
║\x1B[0m    abide            - rest and recover (The Dude approves)    \x1B[35m║
╚══════════════════════════════════════════════════════════════╝\x1B[0m"
        .to_string()
}
