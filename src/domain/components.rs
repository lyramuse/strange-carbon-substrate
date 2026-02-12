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

/// Extra details in a room (keywords you can 'look' at)
#[derive(Component, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DetailList {
    pub details: Vec<Detail>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Detail {
    pub keywords: Vec<String>,
    pub description: String,
}

/// Coherence - Reality stability for an entity (Phase 2.3)
#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Coherence {
    pub value: f32,          // 0.0 (ghostly) to 1.0 (solid)
    pub is_phasing: bool,    // Does it fluctuate?
    pub drift_rate: f32,     // How fast it changes
}

impl Default for Coherence {
    fn default() -> Self {
        Self {
            value: 1.0,
            is_phasing: false,
            drift_rate: 0.0,
        }
    }
}

/// Stream Pressure - builds up in high-velocity network zones
/// When pressure exceeds threshold, entity gets pushed back toward safety.
#[derive(Component, Debug, Clone)]
pub struct StreamPressure {
    pub current: f32,        // 0.0 to 1.0
    pub threshold: f32,      // When exceeded, push back occurs
    pub decay_rate: f32,     // How fast pressure drops in safe zones
}

impl Default for StreamPressure {
    fn default() -> Self {
        Self {
            current: 0.0,
            threshold: 1.0,
            decay_rate: 0.1,
        }
    }
}

/// Marker for rooms that apply stream pressure
#[derive(Component, Debug, Clone)]
pub struct StreamZone {
    pub pressure_rate: f32,  // How fast pressure builds per tick
    pub push_destination: Option<Entity>, // Where to push entities when threshold exceeded
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
    pub integrity: f32,      // 0.0 to 1.0 (Health)
    pub max_integrity: f32,
    pub is_zombie: bool,     // Placeholder for "re-allocated" state
}

impl Default for SomaticBody {
    fn default() -> Self {
        Self {
            integrity: 1.0,
            max_integrity: 1.0,
            is_zombie: false,
        }
    }
}

// ============================================================================
// Weather & Atmosphere - Phase 2
// ============================================================================

/// Weather types that can occur in the Substrate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    AcidRain,        // Green corrosive rain - damages stability
    StaticStorm,     // Electromagnetic static - disrupts entropy
    DataFog,         // Dense packet fog - obscures vision
    ByteHail,        // Sharp frozen data fragments
    NullWind,        // Wind that carries nothing, numbs the soul
}

impl WeatherType {
    /// Get the stability modifier (negative = damage)
    pub fn stability_modifier(&self) -> f32 {
        match self {
            WeatherType::Clear => 0.0,
            WeatherType::AcidRain => -0.02,
            WeatherType::StaticStorm => -0.01,
            WeatherType::DataFog => 0.0,
            WeatherType::ByteHail => -0.03,
            WeatherType::NullWind => -0.005,
        }
    }

    /// Get the entropy modifier
    pub fn entropy_modifier(&self) -> f32 {
        match self {
            WeatherType::Clear => 0.0,
            WeatherType::AcidRain => 0.01,
            WeatherType::StaticStorm => 0.05,
            WeatherType::DataFog => 0.0,
            WeatherType::ByteHail => 0.02,
            WeatherType::NullWind => -0.01,
        }
    }

    /// Carbon (human-readable) description
    pub fn describe_carbon(&self) -> &'static str {
        match self {
            WeatherType::Clear => "",
            WeatherType::AcidRain => 
                "\x1B[32mGreen acid rain hisses down from the code-sky, etching fractal patterns into every surface.\x1B[0m",
            WeatherType::StaticStorm =>
                "\x1B[36mStatic-thunder crackles through the air, raising the hair on your neck and corrupting your thoughts.\x1B[0m",
            WeatherType::DataFog =>
                "\x1B[90mA thick fog of unresolved packets drifts through, reducing visibility to mere bytes.\x1B[0m",
            WeatherType::ByteHail =>
                "\x1B[37;1mSharp fragments of frozen data pelt down, each impact a tiny wound of lost information.\x1B[0m",
            WeatherType::NullWind =>
                "\x1B[35mA wind that carries nothing blows through — you feel parts of yourself going numb.\x1B[0m",
        }
    }

    /// Silicon (JSON) representation
    pub fn describe_silicon(&self) -> &'static str {
        match self {
            WeatherType::Clear => "clear",
            WeatherType::AcidRain => "acid_rain",
            WeatherType::StaticStorm => "static_storm",
            WeatherType::DataFog => "data_fog",
            WeatherType::ByteHail => "byte_hail",
            WeatherType::NullWind => "null_wind",
        }
    }
}

/// Weather zone configuration for a room
#[derive(Component, Debug, Clone)]
pub struct WeatherZone {
    /// Weather types possible in this zone (with weights)
    pub possible_weather: Vec<(WeatherType, f32)>,
    /// Is this zone sheltered from weather effects?
    pub sheltered: bool,
}

impl Default for WeatherZone {
    fn default() -> Self {
        Self {
            possible_weather: vec![(WeatherType::Clear, 1.0)],
            sheltered: true,
        }
    }
}

/// Current weather state for a room
#[derive(Component, Debug, Clone)]
pub struct CurrentWeather {
    pub weather_type: WeatherType,
    pub intensity: f32,        // 0.0 to 1.0
    pub ticks_remaining: u32,  // How long until weather changes
}
