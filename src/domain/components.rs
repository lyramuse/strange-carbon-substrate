// Domain Components - The building blocks of the Substrate

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

/// Client type - Carbon (human) or Silicon (AI agent)
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub enum ClientType {
    Carbon,
    Silicon,
}

/// Network connection for a client
#[derive(Component)]
pub struct NetworkClient {
    pub addr: SocketAddr,
    pub tx: mpsc::UnboundedSender<String>,
}

/// Core identity within the Substrate
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct SubstrateIdentity {
    pub uuid: String,
    pub name: String,
    pub entropy: f32,
    pub stability: f32,
}

/// Admin permission marker - the right to torment, promote, shift
#[derive(Component, Debug, Clone)]
pub struct AdminPermission;

/// Link between two entities (for phase shifting between avatars)
#[derive(Component, Debug, Clone)]
pub struct AdminLink {
    pub partner: Entity,
}

/// Marker for non-player entities
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct NonPlayer;

/// Mobile entity (NPC) with descriptions
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Mob {
    pub short_desc: String,
    pub long_desc: String,
}

/// A room in the world
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Room {
    pub title: String,
    pub description: String,
}

/// Location component - which room an entity is in
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Location(pub Entity);

/// Exits from a room
#[derive(Component, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Exits {
    pub north: Option<Entity>,
    pub south: Option<Entity>,
    pub east: Option<Entity>,
    pub west: Option<Entity>,
    pub up: Option<Entity>,
    pub down: Option<Entity>,
}

impl Exits {
    pub fn get(&self, direction: &str) -> Option<Entity> {
        match direction {
            "north" | "n" => self.north,
            "south" | "s" => self.south,
            "east" | "e" => self.east,
            "west" | "w" => self.west,
            "up" | "u" => self.up,
            "down" | "d" => self.down,
            _ => None,
        }
    }
}

/// An item that can be picked up
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

/// Marker for entities that can hold items
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Inventory;

/// Purgatory state - the velvet chains
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct PurgatoryState {
    pub penance: f32,
    pub tormentor: String,
}

/// Physical body state
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct SomaticBody {
    pub integrity: f32,
    pub is_zombie: bool,
}
