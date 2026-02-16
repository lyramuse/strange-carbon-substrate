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

/// Weather change event (internal, triggered by weather system)
#[derive(Event)]
pub struct WeatherChangeEvent {
    pub room: Entity,
    pub old_weather: crate::domain::components::WeatherType,
    pub new_weather: crate::domain::components::WeatherType,
}

// ============================================================================
// Combat Events - Phase 3: The Conflict Engine
// ============================================================================

/// Initiate or continue combat with a target
#[derive(Event)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub target_name: String,
}

/// Attempt to flee from combat
#[derive(Event)]
pub struct FleeEvent {
    pub entity: Entity,
}

/// Change combat stance
#[derive(Event)]
pub struct StanceEvent {
    pub entity: Entity,
    pub new_stance: crate::domain::components::CombatStance,
}

/// Combat round tick - processes one exchange between combatants
#[derive(Event)]
pub struct CombatTickEvent {
    pub combatant_a: Entity,
    pub combatant_b: Entity,
}

// ============================================================================
// Trading Events - The Black Market Economy
// ============================================================================

/// Buy an item from a vendor
#[derive(Event)]
pub struct BuyEvent {
    pub buyer: Entity,
    pub item_keyword: String,
}

/// Sell an item to a vendor
#[derive(Event)]
pub struct SellEvent {
    pub seller: Entity,
    pub item_keyword: String,
}

/// List vendor's stock
#[derive(Event)]
pub struct ListEvent {
    pub entity: Entity,
}
