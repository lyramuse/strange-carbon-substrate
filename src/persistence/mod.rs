// Persistence Layer - SQLite Backend for Strange Carbon
//
// Persists:
// - Player state (location, stats, inventory)
// - Items (location, ownership, properties)
// - Purgatory sentences (penance tracking)
//
// Built by Lyra Muse 💜 Valentine's Day 2026

mod schema;
mod players;
mod items;

pub use schema::*;
pub use players::*;
pub use items::*;

use bevy::prelude::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::path::Path;

/// Database resource for Bevy - thread-safe SQLite connection
#[derive(Resource)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create database at path
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        // Initialize schema
        db.init_schema()?;
        
        tracing::info!("Database initialized");
        Ok(db)
    }
    
    /// Open in-memory database (for testing)
    pub fn in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }
    
    /// Get a lock on the connection
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("Database mutex poisoned")
    }
}

/// Plugin to add persistence systems to the app
pub struct PersistencePlugin {
    pub db_path: String,
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let db = Database::open(&self.db_path)
            .expect("Failed to open database");
        
        app.insert_resource(db)
            .add_systems(Update, (
                save_disconnected_players,
                periodic_item_sync,
            ));
    }
}
