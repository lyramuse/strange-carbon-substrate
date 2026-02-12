// Domain Events - Things that happen in the Substrate

use bevy::prelude::*;
use std::net::SocketAddr;
use tokio::sync::mpsc;

// ============================================================================
// Network Events - Infrastructure layer, but needed by domain
// ============================================================================

#[derive(Event)]
pub enum NetworkEvent {
    Connected {
        addr: SocketAddr,
        tx: mpsc::UnboundedSender<String>,
    },
    Disconnected {
        addr: SocketAddr,
    },
    Input {
        addr: SocketAddr,
        text: String,
    },
}

// ============================================================================
// Game Events - Pure domain events
// ============================================================================

/// Request to look at the room or a specific target
#[derive(Event)]
pub struct LookEvent {
    pub entity: Entity,
    pub target: Option<String>,
}

/// Request to move in a direction
#[derive(Event)]
pub struct MoveEvent {
    pub entity: Entity,
    pub direction: String,
}

/// Say or emote communication
#[derive(Event)]
pub struct CommunicationEvent {
    pub sender: Entity,
    pub message: String,
    pub is_emote: bool,
}

/// Item interaction (get, drop)
#[derive(Event)]
pub struct ActionEvent {
    pub entity: Entity,
    pub action: String,
    pub target: String,
}

/// Utility commands (score, who, promote, etc)
#[derive(Event)]
pub struct UtilityEvent {
    pub entity: Entity,
    pub command: String,
    pub args: String,
}

/// Admin torment action
#[derive(Event)]
pub struct TormentEvent {
    pub victim: Entity,
    pub tormentor: Entity,
    pub intensity: f32,
    pub description: String,
}

/// Phase shift between linked entities
#[derive(Event)]
pub struct ShiftEvent {
    pub entity: Entity,
}
