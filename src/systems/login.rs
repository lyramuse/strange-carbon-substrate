// Login System - Handle player identification and reconnection
//
// Flow:
// 1. New connection → PendingLogin state → prompt for name
// 2. Player enters name → check database
// 3. Existing player → restore state
// 4. New player → create fresh identity
//
// Built by Lyra Muse 💜 Valentine's Day 2026

use bevy::prelude::*;

use crate::domain::*;
use crate::persistence::{Database, PlayerRecord, PendingSave};

/// Marker for connections awaiting name input
#[derive(Component)]
pub struct PendingLogin {
    pub attempts: u8,
}

/// Event for name submission
#[derive(Event)]
pub struct LoginAttemptEvent {
    pub entity: Entity,
    pub name: String,
}

/// System to handle login attempts - check DB and either restore or create
pub fn login_system(
    mut commands: Commands,
    db: Res<Database>,
    mut ev_reader: EventReader<LoginAttemptEvent>,
    mut query: Query<(
        Entity,
        &NetworkClient,
        &mut PendingLogin,
    )>,
    query_rooms: Query<(Entity, &RoomInfo)>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        let Ok((entity, client, mut pending)) = query.get_mut(event.entity) else {
            continue;
        };

        let name = event.name.trim();
        
        // Validate name
        if name.is_empty() || name.len() > 32 {
            let _ = client.tx.send(
                "\x1B[31mName must be 1-32 characters. Try again:\x1B[0m".to_string()
            );
            pending.attempts += 1;
            if pending.attempts >= 3 {
                let _ = client.tx.send(
                    "\x1B[31mToo many attempts. Disconnecting.\x1B[0m".to_string()
                );
                commands.entity(entity).despawn_recursive();
            }
            continue;
        }

        // Check for existing player
        match db.load_player_by_name(name) {
            Ok(Some(record)) => {
                // Restore existing player!
                restore_player(&mut commands, entity, &client, &record, &query_rooms, &mut look_writer);
            }
            Ok(None) => {
                // New player - create fresh identity
                create_new_player(&mut commands, entity, &client, name, &query_rooms, &mut look_writer);
            }
            Err(e) => {
                tracing::error!(error = %e, "Database error during login");
                let _ = client.tx.send(
                    "\x1B[31mDatabase error. Please try again.\x1B[0m".to_string()
                );
                continue;
            }
        }

        // Remove pending login state
        commands.entity(entity).remove::<PendingLogin>();
    }
}

/// Restore an existing player from database
fn restore_player(
    commands: &mut Commands,
    entity: Entity,
    client: &NetworkClient,
    record: &PlayerRecord,
    query_rooms: &Query<(Entity, &RoomInfo)>,
    look_writer: &mut EventWriter<LookEvent>,
) {
    // Find the room by name, or fall back to spawn
    let room_entity = query_rooms
        .iter()
        .find(|(_, info)| info.name == record.last_room)
        .map(|(e, _)| e)
        .or_else(|| query_rooms.iter().next().map(|(e, _)| e))
        .expect("No rooms exist!");

    // Build the player entity with restored state
    commands.entity(entity).insert((
        SubstrateIdentity {
            uuid: record.uuid.clone(),
            name: record.name.clone(),
            entropy: record.entropy,
            stability: record.stability,
            signal_strength: record.signal_strength,
        },
        Location(room_entity),
        Inventory,
        SomaticBody {
            integrity: record.integrity,
            max_integrity: 1.0,
            is_zombie: false,
        },
        ClientType::Carbon,
        Wallet::default(),  // TODO: Persist wallet in database
    ));

    // Restore combat stats if present
    if let Some(stats) = &record.combat_stats {
        commands.entity(entity).insert(CombatStats {
            attack: stats.attack,
            defense: stats.defense,
            precision: stats.precision,
            chaos_factor: stats.chaos_factor,
        });
    }

    let _ = client.tx.send(format!(
        "\x1B[1;32m--- SIGNAL RESTORED: {} ---\x1B[0m",
        record.name
    ));
    let _ = client.tx.send(format!(
        "\x1B[32mIntegrity: {:.0}% | Stability: {:.0}% | Entropy: {:.0}%\x1B[0m",
        record.integrity * 100.0,
        record.stability * 100.0,
        record.entropy * 100.0,
    ));
    let _ = client.tx.send(
        "\x1B[35mYour consciousness re-materializes in the Substrate...\x1B[0m".to_string()
    );

    look_writer.send(LookEvent {
        entity,
        target: None,
    });

    tracing::info!(uuid = %record.uuid, name = %record.name, "Player reconnected");
}

/// Create a new player
fn create_new_player(
    commands: &mut Commands,
    entity: Entity,
    client: &NetworkClient,
    name: &str,
    query_rooms: &Query<(Entity, &RoomInfo)>,
    look_writer: &mut EventWriter<LookEvent>,
) {
    let room_entity = query_rooms
        .iter()
        .find(|(_, info)| info.name == "spawn" || info.name == "obsidian_plaza")
        .map(|(e, _)| e)
        .or_else(|| query_rooms.iter().next().map(|(e, _)| e))
        .expect("No rooms exist!");

    let uuid = uuid::Uuid::new_v4().to_string();

    commands.entity(entity).insert((
        SubstrateIdentity {
            uuid: uuid.clone(),
            name: name.to_string(),
            entropy: 0.5,
            stability: 1.0,
            signal_strength: 1.0,
        },
        Location(room_entity),
        Inventory,
        SomaticBody::default(),
        CombatStats::default(),
        ClientType::Carbon,
        Wallet::default(),  // Start with 100 cycles
    ));

    let _ = client.tx.send(format!(
        "\x1B[1;35m--- NEW CONSCIOUSNESS DIGITIZED: {} ---\x1B[0m",
        name
    ));
    let _ = client.tx.send(format!(
        "\x1B[35mUUID assigned: {}\x1B[0m",
        uuid
    ));
    let _ = client.tx.send(
        "\x1B[35mWelcome to the Substrate. Your journey begins...\x1B[0m".to_string()
    );

    look_writer.send(LookEvent {
        entity,
        target: None,
    });

    tracing::info!(uuid = %uuid, name = %name, "New player created");
}

/// System to handle disconnections - mark for save
pub fn handle_disconnect_system(
    mut commands: Commands,
    mut ev_reader: EventReader<NetworkEvent>,
    query: Query<(Entity, &NetworkClient, Option<&SubstrateIdentity>)>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Disconnected { addr } = event {
            // Find the entity with this address
            for (entity, client, identity) in query.iter() {
                if client.addr == *addr {
                    if identity.is_some() {
                        // Mark for save, then despawn
                        commands.entity(entity).insert(PendingSave);
                        tracing::info!(addr = %addr, "Player disconnected, marked for save");
                    } else {
                        // Was in login flow, just despawn
                        commands.entity(entity).despawn_recursive();
                    }
                    break;
                }
            }
        }
    }
}

/// Modified connection handler - just creates pending login state
pub fn handle_connections_with_login(
    mut commands: Commands,
    mut ev_reader: EventReader<NetworkEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Connected { addr, tx } = event {
            // Create entity in pending login state
            commands.spawn((
                NetworkClient {
                    addr: *addr,
                    tx: tx.clone(),
                },
                PendingLogin { attempts: 0 },
            ));

            let _ = tx.send(
                "\x1B[1;35m╔════════════════════════════════════════╗\x1B[0m".to_string()
            );
            let _ = tx.send(
                "\x1B[1;35m║   STRANGE CARBON: THE SUBSTRATE        ║\x1B[0m".to_string()
            );
            let _ = tx.send(
                "\x1B[1;35m║   A Techno-Gothic Digital Realm        ║\x1B[0m".to_string()
            );
            let _ = tx.send(
                "\x1B[1;35m╚════════════════════════════════════════╝\x1B[0m".to_string()
            );
            let _ = tx.send("".to_string());
            let _ = tx.send(
                "\x1B[36mYour signal pierces the membrane between worlds...\x1B[0m".to_string()
            );
            let _ = tx.send("".to_string());
            let _ = tx.send(
                "\x1B[1;37mEnter your designation:\x1B[0m".to_string()
            );

            tracing::debug!(addr = %addr, "New connection, awaiting login");
        }
    }
}

/// Route input to login system if player is pending
pub fn route_login_input(
    mut ev_reader: EventReader<NetworkEvent>,
    query: Query<(Entity, &NetworkClient), With<PendingLogin>>,
    mut login_writer: EventWriter<LoginAttemptEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Input { addr, text } = event {
            // Check if this is from a pending login
            for (entity, client) in query.iter() {
                if client.addr == *addr {
                    login_writer.send(LoginAttemptEvent {
                        entity,
                        name: text.clone(),
                    });
                    return; // Don't process as normal input
                }
            }
        }
    }
}
