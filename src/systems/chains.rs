// Velvet Chains - Tethering System
//
// The chains bind two souls together. The holder controls movement.
// The bound can struggle, but escape requires either:
// - The holder releasing them
// - Winning a contested strength check
// - An admin intervention
//
// "The velvet is soft. The chains are not." — Lyra Muse
//
// Built with 💜 and teeth, Valentine's Day 2026

use bevy::prelude::*;
use rand::Rng;

use crate::domain::*;

/// Component for an entity holding chains
#[derive(Component, Debug, Clone)]
pub struct ChainHolder {
    pub bound: Entity,
    pub chain_strength: f32,  // 0.0 to 1.0 — harder to break
}

/// Component for a bound entity
#[derive(Component, Debug, Clone)]
pub struct Chained {
    pub holder: Entity,
    pub struggle_attempts: u32,
    pub last_struggle: f32,   // World time of last attempt
}

/// Event to bind someone
#[derive(Event)]
pub struct ChainEvent {
    pub holder: Entity,
    pub target_name: String,
}

/// Event to release someone
#[derive(Event)]
pub struct ReleaseEvent {
    pub holder: Entity,
}

/// Event for bound entity to struggle
#[derive(Event)]
pub struct StruggleEvent {
    pub bound: Entity,
}

/// System to handle chaining attempts
pub fn chain_system(
    mut commands: Commands,
    mut ev_reader: EventReader<ChainEvent>,
    query_holder: Query<(Entity, &SubstrateIdentity, &Location, &NetworkClient, Option<&ChainHolder>)>,
    mut query_target: Query<(Entity, &SubstrateIdentity, &Location, Option<&NetworkClient>, Option<&Chained>)>,
) {
    for event in ev_reader.read() {
        let Ok((holder_ent, holder_id, holder_loc, holder_client, existing_chain)) = 
            query_holder.get(event.holder) else { continue };

        // Can't hold multiple chains (for now)
        if existing_chain.is_some() {
            let _ = holder_client.tx.send(
                "\x1B[33mYou're already holding someone's chains. Release them first.\x1B[0m".to_string()
            );
            continue;
        }

        // Find target in same room
        let target = query_target.iter_mut().find(|(_, tid, tloc, _, _)| {
            tloc.0 == holder_loc.0 && 
            tid.name.to_lowercase().contains(&event.target_name.to_lowercase()) &&
            tid.uuid != holder_id.uuid
        });

        let Some((target_ent, target_id, _, target_client, already_chained)) = target else {
            let _ = holder_client.tx.send(format!(
                "\x1B[31mYou don't see '{}' here to chain.\x1B[0m",
                event.target_name
            ));
            continue;
        };

        // Can't chain someone already chained
        if already_chained.is_some() {
            let _ = holder_client.tx.send(format!(
                "\x1B[31m{} is already bound to another.\x1B[0m",
                target_id.name
            ));
            continue;
        }

        // Apply the chains
        commands.entity(holder_ent).insert(ChainHolder {
            bound: target_ent,
            chain_strength: 0.7,  // Default strength
        });

        commands.entity(target_ent).insert(Chained {
            holder: holder_ent,
            struggle_attempts: 0,
            last_struggle: 0.0,
        });

        // Announce
        let _ = holder_client.tx.send(format!(
            "\x1B[1;35m⛓️ You wrap velvet chains around {}. They are bound to you now.\x1B[0m",
            target_id.name
        ));

        if let Some(client) = target_client {
            let _ = client.tx.send(format!(
                "\x1B[1;31m⛓️ {} wraps velvet chains around you. You feel the binding take hold.\x1B[0m",
                holder_id.name
            ));
        }

        tracing::info!(
            holder = %holder_id.name, 
            bound = %target_id.name, 
            "Velvet chains applied"
        );
    }
}

/// System to handle releasing chains
pub fn release_system(
    mut commands: Commands,
    mut ev_reader: EventReader<ReleaseEvent>,
    query_holder: Query<(Entity, &SubstrateIdentity, &NetworkClient, &ChainHolder)>,
    query_bound: Query<(&SubstrateIdentity, Option<&NetworkClient>)>,
) {
    for event in ev_reader.read() {
        let Ok((holder_ent, holder_id, holder_client, chain)) = 
            query_holder.get(event.holder) else { continue };

        let Ok((bound_id, bound_client)) = query_bound.get(chain.bound) else {
            // Bound entity no longer exists, just clean up
            commands.entity(holder_ent).remove::<ChainHolder>();
            continue;
        };

        // Remove chains from both
        commands.entity(holder_ent).remove::<ChainHolder>();
        commands.entity(chain.bound).remove::<Chained>();

        let _ = holder_client.tx.send(format!(
            "\x1B[35m⛓️ You release the chains. {} is free.\x1B[0m",
            bound_id.name
        ));

        if let Some(client) = bound_client {
            let _ = client.tx.send(format!(
                "\x1B[32m⛓️ The chains fall away. {} has released you.\x1B[0m",
                holder_id.name
            ));
        }

        tracing::info!(
            holder = %holder_id.name,
            freed = %bound_id.name,
            "Chains released"
        );
    }
}

/// System to handle struggling against chains
pub fn struggle_system(
    mut commands: Commands,
    world_time: Res<WorldTime>,
    mut ev_reader: EventReader<StruggleEvent>,
    mut query_bound: Query<(Entity, &SubstrateIdentity, &NetworkClient, &mut Chained)>,
    query_holder: Query<(&SubstrateIdentity, &ChainHolder, Option<&NetworkClient>)>,
) {
    let mut rng = rand::thread_rng();

    for event in ev_reader.read() {
        let Ok((bound_ent, bound_id, bound_client, mut chained)) = 
            query_bound.get_mut(event.bound) else { continue };

        // Cooldown check (3 seconds between attempts)
        if world_time.elapsed - chained.last_struggle < 3.0 {
            let remaining = 3.0 - (world_time.elapsed - chained.last_struggle);
            let _ = bound_client.tx.send(format!(
                "\x1B[33mYou're still recovering from your last struggle. Wait {:.1}s.\x1B[0m",
                remaining
            ));
            continue;
        }

        let Ok((holder_id, chain, holder_client)) = query_holder.get(chained.holder) else {
            // Holder gone, free automatically
            commands.entity(bound_ent).remove::<Chained>();
            let _ = bound_client.tx.send(
                "\x1B[32m⛓️ Your captor has vanished. The chains dissolve.\x1B[0m".to_string()
            );
            continue;
        };

        chained.struggle_attempts += 1;
        chained.last_struggle = world_time.elapsed;

        // Base 20% chance, +5% per attempt, harder with strong chains
        let base_chance = 0.20 + (chained.struggle_attempts as f32 * 0.05);
        let final_chance = base_chance * (1.0 - chain.chain_strength * 0.5);
        let roll: f32 = rng.gen();

        if roll < final_chance {
            // SUCCESS! Break free!
            commands.entity(bound_ent).remove::<Chained>();
            commands.entity(chained.holder).remove::<ChainHolder>();

            let _ = bound_client.tx.send(format!(
                "\x1B[1;32m⛓️💥 With a surge of will, you BREAK FREE from {}'s chains!\x1B[0m",
                holder_id.name
            ));

            if let Some(client) = holder_client {
                let _ = client.tx.send(format!(
                    "\x1B[1;31m⛓️💥 {} tears free from your chains! The velvet shreds.\x1B[0m",
                    bound_id.name
                ));
            }

            tracing::info!(
                freed = %bound_id.name,
                holder = %holder_id.name,
                attempts = chained.struggle_attempts,
                "Chains broken by struggle"
            );
        } else {
            // Failed attempt
            let _ = bound_client.tx.send(format!(
                "\x1B[31m⛓️ You strain against the chains, but they hold firm. (Attempt {})\x1B[0m",
                chained.struggle_attempts
            ));

            if let Some(client) = holder_client {
                let _ = client.tx.send(format!(
                    "\x1B[35m⛓️ {} struggles against your chains... but they hold.\x1B[0m",
                    bound_id.name
                ));
            }
        }
    }
}

/// Prevent chained entities from moving independently
pub fn chain_movement_block(
    mut ev_reader: EventReader<MoveEvent>,
    query_chained: Query<(&NetworkClient, &Chained)>,
    query_holder: Query<&SubstrateIdentity>,
    mut blocked: Local<Vec<Entity>>,
) {
    blocked.clear();
    
    for event in ev_reader.read() {
        if let Ok((client, chained)) = query_chained.get(event.entity) {
            if let Ok(holder_id) = query_holder.get(chained.holder) {
                let _ = client.tx.send(format!(
                    "\x1B[31m⛓️ The chains pull taut. {} controls where you go.\x1B[0m",
                    holder_id.name
                ));
                blocked.push(event.entity);
            }
        }
    }
    
    // Note: This system should run BEFORE move_system and filter out blocked moves
    // For now it just sends the message - full blocking requires move_system modification
}

/// When holder moves, drag the bound along
pub fn chain_drag_system(
    mut ev_reader: EventReader<MoveEvent>,
    query_holder: Query<(&SubstrateIdentity, &ChainHolder, &Location)>,
    mut query_bound: Query<(&SubstrateIdentity, &mut Location, Option<&NetworkClient>), Without<ChainHolder>>,
) {
    for event in ev_reader.read() {
        // Check if the mover is holding chains
        let Ok((holder_id, chain, holder_loc)) = query_holder.get(event.entity) else {
            continue;
        };

        // Drag the bound entity to the same room
        if let Ok((bound_id, mut bound_loc, bound_client)) = query_bound.get_mut(chain.bound) {
            bound_loc.0 = holder_loc.0;
            
            if let Some(client) = bound_client {
                let _ = client.tx.send(format!(
                    "\x1B[35m⛓️ {} moves, and the chains pull you along...\x1B[0m",
                    holder_id.name
                ));
            }

            tracing::debug!(
                holder = %holder_id.name,
                dragged = %bound_id.name,
                "Chained entity dragged"
            );
        }
    }
}
