// Database Schema - Strange Carbon: The Substrate
//
// Three core tables:
// - players: Identity, location, stats, inventory
// - items: World objects with ownership and location
// - purgatory: Sentence tracking for the damned

use super::Database;

impl Database {
    /// Initialize all tables
    pub fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn();
        
        // Players table - substrate identities
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS players (
                uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                client_type TEXT NOT NULL DEFAULT 'Carbon',
                
                -- Location (stored as room entity index, mapped on load)
                last_room TEXT NOT NULL DEFAULT 'spawn',
                
                -- Substrate Identity stats
                stability REAL NOT NULL DEFAULT 1.0,
                entropy REAL NOT NULL DEFAULT 0.5,
                signal_strength REAL NOT NULL DEFAULT 1.0,
                
                -- Somatic Body stats  
                integrity REAL NOT NULL DEFAULT 1.0,
                
                -- Combat stats (JSON blob for flexibility)
                combat_stats TEXT,
                
                -- Inventory (JSON array of item UUIDs)
                inventory TEXT NOT NULL DEFAULT '[]',
                
                -- Timestamps
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                total_playtime_seconds INTEGER NOT NULL DEFAULT 0
            );
            
            CREATE INDEX IF NOT EXISTS idx_players_name ON players(name);
        "#)?;
        
        // Items table - world objects
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS items (
                uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                keywords TEXT NOT NULL DEFAULT '[]',
                
                -- Location: either room_id OR owner_uuid, never both
                room_id TEXT,
                owner_uuid TEXT,
                
                -- Item properties (type-specific data as JSON)
                item_type TEXT NOT NULL DEFAULT 'misc',
                properties TEXT NOT NULL DEFAULT '{}',
                
                -- Flags
                is_takeable INTEGER NOT NULL DEFAULT 1,
                is_visible INTEGER NOT NULL DEFAULT 1,
                
                -- Timestamps
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                
                FOREIGN KEY (owner_uuid) REFERENCES players(uuid)
            );
            
            CREATE INDEX IF NOT EXISTS idx_items_room ON items(room_id);
            CREATE INDEX IF NOT EXISTS idx_items_owner ON items(owner_uuid);
        "#)?;
        
        // Purgatory table - sentence tracking
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS purgatory (
                player_uuid TEXT PRIMARY KEY,
                penance REAL NOT NULL DEFAULT 0.0,
                crimes TEXT NOT NULL DEFAULT '[]',
                entry_time TEXT NOT NULL DEFAULT (datetime('now')),
                release_time TEXT,
                
                FOREIGN KEY (player_uuid) REFERENCES players(uuid)
            );
        "#)?;
        
        // World state table - misc persistent world data
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS world_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        "#)?;
        
        tracing::debug!("Database schema initialized");
        Ok(())
    }
    
    /// Reset database (for testing/development)
    #[allow(dead_code)]
    pub fn reset(&self) -> anyhow::Result<()> {
        let conn = self.conn();
        conn.execute_batch(r#"
            DROP TABLE IF EXISTS purgatory;
            DROP TABLE IF EXISTS items;
            DROP TABLE IF EXISTS players;
            DROP TABLE IF EXISTS world_state;
        "#)?;
        drop(conn);
        self.init_schema()
    }
}
