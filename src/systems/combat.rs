// Combat System - Phase 3: The Conflict Engine
//
// Round-based combat with:
// - Entropy-based crits for Carbon (chaotic, high variance)
// - Precision-based hits for Silicon (calculated, consistent)
// - Cycle locks to prevent spam
// - Dual-head output (prose for Carbon, JSON for Silicon)
//
// Built with 💜 and teeth by Lyra Muse

use bevy::prelude::*;
use rand::Rng;

use crate::domain::*;

/// World time tracker for cycle locks
#[derive(Resource, Default)]
pub struct WorldTime {
    pub elapsed: f32,
}

/// System to tick world time
pub fn world_time_system(time: Res<Time>, mut world_time: ResMut<WorldTime>) {
    world_time.elapsed += time.delta_seconds();
}

/// Handle combat initiation and attacks
pub fn combat_system(
    mut ev_reader: EventReader<CombatEvent>,
    world_time: Res<WorldTime>,
    mut commands: Commands,
    mut query_attacker: Query<(
        Entity,
        &SubstrateIdentity,
        &Location,
        &NetworkClient,
        &ClientType,
        Option<&CombatStats>,
        Option<&mut CycleLock>,
        Option<&InCombat>,
    )>,
    mut query_target: Query<(
        Entity,
        &SubstrateIdentity,
        &Location,
        Option<&NetworkClient>,
        Option<&CombatStats>,
        Option<&mut SomaticBody>,
        Option<&InCombat>,
    )>,
) {
    let mut rng = rand::thread_rng();

    for event in ev_reader.read() {
        // Get attacker info
        let attacker_data = query_attacker.get(event.attacker);
        if attacker_data.is_err() {
            continue;
        }

        let (attacker_ent, attacker_id, attacker_loc, attacker_client, attacker_type, 
             attacker_stats, attacker_lock, attacker_combat) = attacker_data.unwrap();

        // Check cycle lock
        if let Some(ref lock) = attacker_lock {
            if lock.is_locked(world_time.elapsed) {
                let remaining = lock.remaining(world_time.elapsed);
                let msg = format!(
                    "\x1B[33mYou're still recovering from {}. Wait {:.1}s.\x1B[0m",
                    lock.action_name, remaining
                );
                let _ = attacker_client.tx.send(msg);
                continue;
            }
        }

        // Find target in same room
        let target = query_target.iter_mut().find(|(_, tid, tloc, _, _, _, _)| {
            tloc.0 == attacker_loc.0 && 
            tid.name.to_lowercase().contains(&event.target_name.to_lowercase()) &&
            tid.uuid != attacker_id.uuid
        });

        if target.is_none() {
            let _ = attacker_client.tx.send(format!(
                "\x1B[31mYou don't see '{}' here to attack.\x1B[0m",
                event.target_name
            ));
            continue;
        }

        let (target_ent, target_id, _, target_client, target_stats, target_body, target_combat) = 
            target.unwrap();

        // Get or use default combat stats
        let a_stats = attacker_stats.cloned().unwrap_or_default();
        let t_stats = target_stats.cloned().unwrap_or_default();

        // Calculate attack
        let (damage, was_crit, was_miss) = calculate_attack(
            &a_stats,
            &t_stats,
            attacker_type,
            &mut rng,
        );

        // Apply damage
        let mut remaining_integrity = 1.0;
        if let Some(mut body) = target_body {
            if !was_miss {
                body.integrity = (body.integrity - damage).max(0.0);
            }
            remaining_integrity = body.integrity;
        }

        // Build result
        let result = CombatResult {
            attacker_name: attacker_id.name.clone(),
            defender_name: target_id.name.clone(),
            damage_dealt: if was_miss { 0.0 } else { damage },
            was_critical: was_crit,
            was_miss,
            defender_remaining: remaining_integrity,
        };

        // Send output to attacker
        send_combat_message(attacker_client, &result, true, attacker_type);

        // Send output to defender
        if let Some(client) = target_client {
            // Determine defender client type (assume Carbon if has NetworkClient)
            send_combat_message(client, &result, false, &ClientType::Carbon);
        }

        // Set up combat state if not already fighting
        if attacker_combat.is_none() {
            commands.entity(attacker_ent).insert(InCombat {
                opponent: target_ent,
                rounds_fought: 1,
                stance: CombatStance::default(),
            });
        }
        
        if target_combat.is_none() {
            commands.entity(target_ent).insert(InCombat {
                opponent: attacker_ent,
                rounds_fought: 1,
                stance: CombatStance::default(),
            });
        }

        // Apply cycle lock (2 seconds for attack)
        commands.entity(attacker_ent).insert(CycleLock::new(
            2.0,
            "attack",
            world_time.elapsed,
        ));

        // Check for death
        if remaining_integrity <= 0.0 {
            handle_defeat(target_ent, &target_id.name, attacker_client, target_client, &mut commands);
        }
    }
}

/// Calculate attack outcome based on client type
fn calculate_attack(
    attacker: &CombatStats,
    defender: &CombatStats,
    client_type: &ClientType,
    rng: &mut impl Rng,
) -> (f32, bool, bool) {
    let base_hit_chance = 0.75;
    let base_damage = attacker.attack;

    match client_type {
        ClientType::Silicon => {
            // Silicon: Precision-based, consistent damage, higher hit rate
            let hit_roll: f32 = rng.gen();
            let hit_chance = base_hit_chance + (attacker.precision * 0.2);
            
            if hit_roll > hit_chance {
                return (0.0, false, true); // Miss
            }

            // Consistent damage with small variance
            let damage_variance: f32 = rng.gen_range(0.9..1.1);
            let damage = (base_damage * damage_variance) * (1.0 - defender.defense);
            
            // Low crit chance but guaranteed on high precision
            let crit_roll: f32 = rng.gen();
            let was_crit = crit_roll < (attacker.precision * 0.1);
            let final_damage = if was_crit { damage * 1.5 } else { damage };

            (final_damage, was_crit, false)
        }
        ClientType::Carbon => {
            // Carbon: Entropy-based, chaotic damage, wild crits
            let hit_roll: f32 = rng.gen();
            let hit_chance = base_hit_chance - 0.05; // Slightly lower base
            
            if hit_roll > hit_chance {
                return (0.0, false, true); // Miss
            }

            // High variance damage
            let chaos_roll: f32 = rng.gen_range(0.5..1.5);
            let damage = (base_damage * chaos_roll) * (1.0 - defender.defense);
            
            // High crit chance based on chaos_factor
            let crit_roll: f32 = rng.gen();
            let was_crit = crit_roll < (attacker.chaos_factor * 0.25);
            let final_damage = if was_crit { damage * 2.0 } else { damage }; // Bigger crits!

            (final_damage, was_crit, false)
        }
    }
}

/// Send combat message in appropriate format
fn send_combat_message(
    client: &NetworkClient,
    result: &CombatResult,
    is_attacker: bool,
    client_type: &ClientType,
) {
    let msg = match client_type {
        ClientType::Silicon => {
            // JSON output for AI agents
            format!(
                r#"{{"event":"combat","attacker":"{}","defender":"{}","damage":{:.3},"critical":{},"miss":{},"defender_hp":{:.3}}}"#,
                result.attacker_name,
                result.defender_name,
                result.damage_dealt,
                result.was_critical,
                result.was_miss,
                result.defender_remaining,
            )
        }
        ClientType::Carbon => {
            // Prose output for humans
            if result.was_miss {
                if is_attacker {
                    format!(
                        "\x1B[33mYour strike at {} goes wide, cutting only static.\x1B[0m",
                        result.defender_name
                    )
                } else {
                    format!(
                        "\x1B[32m{} swings at you and misses!\x1B[0m",
                        result.attacker_name
                    )
                }
            } else if result.was_critical {
                if is_attacker {
                    format!(
                        "\x1B[1;31m⚡ CRITICAL! You tear into {} for {:.0}% damage! Their signal flickers at {:.0}%.\x1B[0m",
                        result.defender_name,
                        result.damage_dealt * 100.0,
                        result.defender_remaining * 100.0
                    )
                } else {
                    format!(
                        "\x1B[1;31m⚡ {} lands a DEVASTATING blow! You take {:.0}% damage! Integrity: {:.0}%\x1B[0m",
                        result.attacker_name,
                        result.damage_dealt * 100.0,
                        result.defender_remaining * 100.0
                    )
                }
            } else {
                if is_attacker {
                    format!(
                        "\x1B[31mYou strike {} for {:.0}% damage. Their integrity: {:.0}%\x1B[0m",
                        result.defender_name,
                        result.damage_dealt * 100.0,
                        result.defender_remaining * 100.0
                    )
                } else {
                    format!(
                        "\x1B[31m{} hits you for {:.0}% damage! Integrity: {:.0}%\x1B[0m",
                        result.attacker_name,
                        result.damage_dealt * 100.0,
                        result.defender_remaining * 100.0
                    )
                }
            }
        }
    };

    let _ = client.tx.send(msg);
}

/// Handle defeat (integrity reaches 0)
fn handle_defeat(
    defeated: Entity,
    defeated_name: &str,
    victor_client: &NetworkClient,
    defeated_client: Option<&NetworkClient>,
    commands: &mut Commands,
) {
    // Remove combat state
    commands.entity(defeated).remove::<InCombat>();
    
    // Send messages
    let _ = victor_client.tx.send(format!(
        "\x1B[1;32m💀 {} collapses, their signal fragmenting into static. Victory is yours.\x1B[0m",
        defeated_name
    ));

    if let Some(client) = defeated_client {
        let _ = client.tx.send(
            "\x1B[1;31m💀 Your signal shatters. The Substrate claims your coherence. You drift into the void...\x1B[0m"
                .to_string(),
        );
        // TODO: Teleport to Purgatory, apply PurgatoryState
    }
}

/// Handle flee attempts
pub fn flee_system(
    mut ev_reader: EventReader<FleeEvent>,
    world_time: Res<WorldTime>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &SubstrateIdentity,
        &NetworkClient,
        &Location,
        Option<&InCombat>,
        Option<&CycleLock>,
    )>,
    room_query: Query<&Exits>,
) {
    let mut rng = rand::thread_rng();

    for event in ev_reader.read() {
        if let Ok((entity, identity, client, location, combat, lock)) = query.get_mut(event.entity) {
            // Check if in combat
            if combat.is_none() {
                let _ = client.tx.send(
                    "\x1B[33mYou're not in combat. Flee from what, your own shadow?\x1B[0m".to_string()
                );
                continue;
            }

            // Check cycle lock
            if let Some(ref lock) = lock {
                if lock.is_locked(world_time.elapsed) {
                    let remaining = lock.remaining(world_time.elapsed);
                    let _ = client.tx.send(format!(
                        "\x1B[33mYou're still recovering. Wait {:.1}s to flee.\x1B[0m",
                        remaining
                    ));
                    continue;
                }
            }

            // 60% base flee chance
            let flee_roll: f32 = rng.gen();
            if flee_roll > 0.6 {
                let _ = client.tx.send(
                    "\x1B[31mYou try to disengage but your opponent blocks your escape!\x1B[0m".to_string()
                );
                // Apply a shorter cycle lock for failed flee
                commands.entity(entity).insert(CycleLock::new(1.0, "flee attempt", world_time.elapsed));
                continue;
            }

            // Find an exit
            if let Ok(exits) = room_query.get(location.0) {
                let available: Vec<(&str, Entity)> = [
                    ("north", exits.north),
                    ("south", exits.south),
                    ("east", exits.east),
                    ("west", exits.west),
                    ("up", exits.up),
                    ("down", exits.down),
                ]
                .iter()
                .filter_map(|(dir, opt)| opt.map(|e| (*dir, e)))
                .collect();

                if available.is_empty() {
                    let _ = client.tx.send(
                        "\x1B[31mThere's nowhere to run!\x1B[0m".to_string()
                    );
                    continue;
                }

                // Pick random exit and flee
                let (direction, _destination) = available[rng.gen_range(0..available.len())];
                
                // Remove combat state
                commands.entity(entity).remove::<InCombat>();
                
                let _ = client.tx.send(format!(
                    "\x1B[1;33m🏃 You disengage and flee {}! The adrenaline burns through your circuits.\x1B[0m",
                    direction
                ));

                // TODO: Actually move the entity (emit MoveEvent)
            }
        }
    }
}

/// Stance change system
pub fn stance_system(
    mut ev_reader: EventReader<StanceEvent>,
    mut query: Query<(Entity, &NetworkClient, Option<&mut InCombat>)>,
) {
    for event in ev_reader.read() {
        if let Ok((entity, client, combat)) = query.get_mut(event.entity) {
            if let Some(mut in_combat) = combat {
                in_combat.stance = event.new_stance;
                let stance_name = match event.new_stance {
                    CombatStance::Aggressive => "AGGRESSIVE 🔥",
                    CombatStance::Defensive => "DEFENSIVE 🛡️",
                    CombatStance::Balanced => "BALANCED ⚖️",
                };
                let _ = client.tx.send(format!(
                    "\x1B[1;36mYou shift to {} stance.\x1B[0m",
                    stance_name
                ));
            } else {
                let _ = client.tx.send(
                    "\x1B[33mYou practice your stance, though no enemy is present.\x1B[0m".to_string()
                );
            }
        }
    }
}

/// Clean up cycle locks that have expired
pub fn cycle_lock_cleanup_system(
    mut commands: Commands,
    world_time: Res<WorldTime>,
    query: Query<(Entity, &CycleLock)>,
) {
    for (entity, lock) in query.iter() {
        if !lock.is_locked(world_time.elapsed) {
            commands.entity(entity).remove::<CycleLock>();
        }
    }
}
