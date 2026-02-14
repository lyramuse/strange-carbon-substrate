// Item Persistence - Save and load world objects
//
// Items can be:
// - In a room (room_id set)
// - In a player's inventory (owner_uuid set)
// - Neither (limbo/destroyed)

use super::Database;
use bevy::prelude::*;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serializable item record for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRecord {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub room_id: Option<String>,
    pub owner_uuid: Option<String>,
    pub item_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub is_takeable: bool,
    pub is_visible: bool,
}

impl Database {
    /// Save an item to the database
    pub fn save_item(&self, item: &ItemRecord) -> anyhow::Result<()> {
        let conn = self.conn();
        
        let keywords_json = serde_json::to_string(&item.keywords)?;
        let properties_json = serde_json::to_string(&item.properties)?;
        
        conn.execute(
            r#"
            INSERT INTO items (
                uuid, name, description, keywords,
                room_id, owner_uuid, item_type, properties,
                is_takeable, is_visible
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(uuid) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                keywords = excluded.keywords,
                room_id = excluded.room_id,
                owner_uuid = excluded.owner_uuid,
                item_type = excluded.item_type,
                properties = excluded.properties,
                is_takeable = excluded.is_takeable,
                is_visible = excluded.is_visible
            "#,
            params![
                item.uuid,
                item.name,
                item.description,
                keywords_json,
                item.room_id,
                item.owner_uuid,
                item.item_type,
                properties_json,
                item.is_takeable as i32,
                item.is_visible as i32,
            ],
        )?;
        
        tracing::debug!(uuid = %item.uuid, name = %item.name, "Item saved");
        Ok(())
    }
    
    /// Load an item by UUID
    pub fn load_item(&self, uuid: &str) -> anyhow::Result<Option<ItemRecord>> {
        let conn = self.conn();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT uuid, name, description, keywords,
                   room_id, owner_uuid, item_type, properties,
                   is_takeable, is_visible
            FROM items WHERE uuid = ?1
            "#
        )?;
        
        let result = stmt.query_row(params![uuid], |row| {
            let keywords_json: String = row.get(3)?;
            let properties_json: String = row.get(7)?;
            
            Ok(ItemRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
                room_id: row.get(4)?,
                owner_uuid: row.get(5)?,
                item_type: row.get(6)?,
                properties: serde_json::from_str(&properties_json).unwrap_or_default(),
                is_takeable: row.get::<_, i32>(8)? != 0,
                is_visible: row.get::<_, i32>(9)? != 0,
            })
        });
        
        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Load all items in a room
    pub fn load_items_in_room(&self, room_id: &str) -> anyhow::Result<Vec<ItemRecord>> {
        let conn = self.conn();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT uuid, name, description, keywords,
                   room_id, owner_uuid, item_type, properties,
                   is_takeable, is_visible
            FROM items WHERE room_id = ?1
            "#
        )?;
        
        let rows = stmt.query_map(params![room_id], |row| {
            let keywords_json: String = row.get(3)?;
            let properties_json: String = row.get(7)?;
            
            Ok(ItemRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
                room_id: row.get(4)?,
                owner_uuid: row.get(5)?,
                item_type: row.get(6)?,
                properties: serde_json::from_str(&properties_json).unwrap_or_default(),
                is_takeable: row.get::<_, i32>(8)? != 0,
                is_visible: row.get::<_, i32>(9)? != 0,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
    
    /// Load all items owned by a player
    pub fn load_player_inventory(&self, player_uuid: &str) -> anyhow::Result<Vec<ItemRecord>> {
        let conn = self.conn();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT uuid, name, description, keywords,
                   room_id, owner_uuid, item_type, properties,
                   is_takeable, is_visible
            FROM items WHERE owner_uuid = ?1
            "#
        )?;
        
        let rows = stmt.query_map(params![player_uuid], |row| {
            let keywords_json: String = row.get(3)?;
            let properties_json: String = row.get(7)?;
            
            Ok(ItemRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
                room_id: row.get(4)?,
                owner_uuid: row.get(5)?,
                item_type: row.get(6)?,
                properties: serde_json::from_str(&properties_json).unwrap_or_default(),
                is_takeable: row.get::<_, i32>(8)? != 0,
                is_visible: row.get::<_, i32>(9)? != 0,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
    
    /// Delete an item
    pub fn delete_item(&self, uuid: &str) -> anyhow::Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM items WHERE uuid = ?1", params![uuid])?;
        tracing::debug!(uuid = %uuid, "Item deleted");
        Ok(())
    }
    
    /// Move item to a room (removes owner)
    pub fn move_item_to_room(&self, uuid: &str, room_id: &str) -> anyhow::Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE items SET room_id = ?1, owner_uuid = NULL WHERE uuid = ?2",
            params![room_id, uuid],
        )?;
        Ok(())
    }
    
    /// Move item to a player's inventory (removes room)
    pub fn move_item_to_player(&self, uuid: &str, player_uuid: &str) -> anyhow::Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE items SET owner_uuid = ?1, room_id = NULL WHERE uuid = ?2",
            params![player_uuid, uuid],
        )?;
        Ok(())
    }
}

/// Marker for items that need syncing to database
#[derive(Component)]
pub struct ItemDirty;

/// Periodic system to sync dirty items to database
pub fn periodic_item_sync(
    mut commands: Commands,
    db: Res<Database>,
    query: Query<(Entity, &crate::domain::Item), With<ItemDirty>>,
    room_query: Query<&crate::domain::RoomInfo>,
) {
    for (entity, item) in query.iter() {
        let room_name = item.location
            .and_then(|loc| room_query.get(loc).ok())
            .map(|r| r.name.clone());
        
        let record = ItemRecord {
            uuid: item.uuid.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            keywords: item.keywords.clone(),
            room_id: room_name,
            owner_uuid: item.owner.clone(),
            item_type: format!("{:?}", item.item_type),
            properties: item.properties.clone(),
            is_takeable: item.is_takeable,
            is_visible: item.is_visible,
        };
        
        if let Err(e) = db.save_item(&record) {
            tracing::error!(error = %e, uuid = %item.uuid, "Failed to save item");
        } else {
            commands.entity(entity).remove::<ItemDirty>();
        }
    }
}
