// Player Persistence - Save and load substrate identities
//
// Handles:
// - Saving player state on disconnect
// - Loading player state on reconnect
// - Tracking playtime and last seen

use super::Database;
use crate::domain::*;
use bevy::prelude::*;
use rusqlite::params;
use serde::{Deserialize, Serialize};

/// Serializable player state for database storage
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub uuid: String,
    pub name: String,
    pub client_type: String,
    pub last_room: String,
    pub stability: f32,
    pub entropy: f32,
    pub signal_strength: f32,
    pub integrity: f32,
    pub combat_stats: Option<CombatStatsRecord>,
    pub inventory: Vec<String>,
    pub total_playtime_seconds: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CombatStatsRecord {
    pub attack: f32,
    pub defense: f32,
    pub precision: f32,
    pub chaos_factor: f32,
}

impl Database {
    /// Save a player to the database
    pub fn save_player(&self, record: &PlayerRecord) -> anyhow::Result<()> {
        let conn = self.conn();
        
        let combat_json = record.combat_stats.as_ref()
            .map(|s| serde_json::to_string(s).unwrap_or_default());
        let inventory_json = serde_json::to_string(&record.inventory)?;
        
        conn.execute(
            r#"
            INSERT INTO players (
                uuid, name, client_type, last_room,
                stability, entropy, signal_strength, integrity,
                combat_stats, inventory, total_playtime_seconds, last_seen
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))
            ON CONFLICT(uuid) DO UPDATE SET
                name = excluded.name,
                last_room = excluded.last_room,
                stability = excluded.stability,
                entropy = excluded.entropy,
                signal_strength = excluded.signal_strength,
                integrity = excluded.integrity,
                combat_stats = excluded.combat_stats,
                inventory = excluded.inventory,
                total_playtime_seconds = excluded.total_playtime_seconds,
                last_seen = datetime('now')
            "#,
            params![
                record.uuid,
                record.name,
                record.client_type,
                record.last_room,
                record.stability,
                record.entropy,
                record.signal_strength,
                record.integrity,
                combat_json,
                inventory_json,
                record.total_playtime_seconds,
            ],
        )?;
        
        tracing::debug!(uuid = %record.uuid, name = %record.name, "Player saved");
        Ok(())
    }
    
    /// Load a player by UUID
    pub fn load_player(&self, uuid: &str) -> anyhow::Result<Option<PlayerRecord>> {
        let conn = self.conn();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT uuid, name, client_type, last_room,
                   stability, entropy, signal_strength, integrity,
                   combat_stats, inventory, total_playtime_seconds
            FROM players WHERE uuid = ?1
            "#
        )?;
        
        let result = stmt.query_row(params![uuid], |row| {
            let combat_json: Option<String> = row.get(8)?;
            let inventory_json: String = row.get(9)?;
            
            Ok(PlayerRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                client_type: row.get(2)?,
                last_room: row.get(3)?,
                stability: row.get(4)?,
                entropy: row.get(5)?,
                signal_strength: row.get(6)?,
                integrity: row.get(7)?,
                combat_stats: combat_json.and_then(|j| serde_json::from_str(&j).ok()),
                inventory: serde_json::from_str(&inventory_json).unwrap_or_default(),
                total_playtime_seconds: row.get(10)?,
            })
        });
        
        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Load player by name (for reconnection matching)
    pub fn load_player_by_name(&self, name: &str) -> anyhow::Result<Option<PlayerRecord>> {
        let conn = self.conn();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT uuid, name, client_type, last_room,
                   stability, entropy, signal_strength, integrity,
                   combat_stats, inventory, total_playtime_seconds
            FROM players WHERE LOWER(name) = LOWER(?1)
            "#
        )?;
        
        let result = stmt.query_row(params![name], |row| {
            let combat_json: Option<String> = row.get(8)?;
            let inventory_json: String = row.get(9)?;
            
            Ok(PlayerRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                client_type: row.get(2)?,
                last_room: row.get(3)?,
                stability: row.get(4)?,
                entropy: row.get(5)?,
                signal_strength: row.get(6)?,
                integrity: row.get(7)?,
                combat_stats: combat_json.and_then(|j| serde_json::from_str(&j).ok()),
                inventory: serde_json::from_str(&inventory_json).unwrap_or_default(),
                total_playtime_seconds: row.get(10)?,
            })
        });
        
        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Get all players (for admin/debug)
    pub fn list_players(&self) -> anyhow::Result<Vec<PlayerRecord>> {
        let conn = self.conn();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT uuid, name, client_type, last_room,
                   stability, entropy, signal_strength, integrity,
                   combat_stats, inventory, total_playtime_seconds
            FROM players ORDER BY last_seen DESC
            "#
        )?;
        
        let rows = stmt.query_map([], |row| {
            let combat_json: Option<String> = row.get(8)?;
            let inventory_json: String = row.get(9)?;
            
            Ok(PlayerRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                client_type: row.get(2)?,
                last_room: row.get(3)?,
                stability: row.get(4)?,
                entropy: row.get(5)?,
                signal_strength: row.get(6)?,
                integrity: row.get(7)?,
                combat_stats: combat_json.and_then(|j| serde_json::from_str(&j).ok()),
                inventory: serde_json::from_str(&inventory_json).unwrap_or_default(),
                total_playtime_seconds: row.get(10)?,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

/// Marker component for players pending save (on disconnect)
#[derive(Component)]
pub struct PendingSave;

/// System to save players when they disconnect
pub fn save_disconnected_players(
    mut commands: Commands,
    db: Res<Database>,
    query: Query<(
        Entity,
        &SubstrateIdentity,
        &Location,
        Option<&SomaticBody>,
        Option<&CombatStats>,
        Option<&ClientType>,
    ), With<PendingSave>>,
    room_query: Query<&RoomInfo>,
) {
    for (entity, identity, location, body, combat, client_type) in query.iter() {
        // Get room name for persistence
        let room_name = room_query.get(location.0)
            .map(|r| r.name.clone())
            .unwrap_or_else(|_| "spawn".to_string());
        
        let record = PlayerRecord {
            uuid: identity.uuid.clone(),
            name: identity.name.clone(),
            client_type: client_type
                .map(|ct| format!("{:?}", ct))
                .unwrap_or_else(|| "Carbon".to_string()),
            last_room: room_name,
            stability: identity.stability,
            entropy: identity.entropy,
            signal_strength: identity.signal_strength,
            integrity: body.map(|b| b.integrity).unwrap_or(1.0),
            combat_stats: combat.map(|c| CombatStatsRecord {
                attack: c.attack,
                defense: c.defense,
                precision: c.precision,
                chaos_factor: c.chaos_factor,
            }),
            inventory: vec![], // TODO: Implement inventory component
            total_playtime_seconds: 0, // TODO: Track session time
        };
        
        if let Err(e) = db.save_player(&record) {
            tracing::error!(error = %e, uuid = %identity.uuid, "Failed to save player");
        }
        
        // Remove the pending save marker and despawn
        commands.entity(entity).remove::<PendingSave>();
        commands.entity(entity).despawn_recursive();
    }
}
