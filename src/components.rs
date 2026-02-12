use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

#[derive(Component, Debug, Serialize, Deserialize)]
pub enum ClientType {
    Carbon,  // Human (Telnet/Prose)
    Silicon, // AI (JSON/Data)
}

#[derive(Component)]
pub struct NetworkClient {
    pub addr: SocketAddr,
    pub tx: mpsc::UnboundedSender<String>,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct SubstrateIdentity {
    pub name: String,
    pub entropy: f32,
    pub stability: f32,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Room {
    pub title: String,
    pub description: String,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Location(pub Entity);

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Exits {
    pub north: Option<Entity>,
    pub south: Option<Entity>,
    pub east: Option<Entity>,
    pub west: Option<Entity>,
    pub up: Option<Entity>,
    pub down: Option<Entity>,
}
