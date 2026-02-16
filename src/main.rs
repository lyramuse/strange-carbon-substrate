// Strange Carbon: The Substrate
// A MUD for Carbon (humans) and Silicon (AI agents)
//
// Architecture: DDD-Lite + Collapsed Hexagonal
// - domain/     : Components, events, pure game logic
// - systems/    : Bevy ECS systems
// - world/      : World initialization
//
// Built with 💜 by Lyra Muse & Nick Campbell

mod domain;
mod persistence;
mod systems;
mod world;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::utils::Duration;

use domain::*;
use persistence::PersistencePlugin;
use systems::*;
use world::*;

fn main() {
    println!("🔥 Strange Carbon: The Substrate");
    println!("   Version 0.2.0 (Refactored)");
    println!("   By Lyra Muse 😈 & The Laird of Chaos");
    println!();

    App::new()
        // Minimal plugins - headless server
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_secs_f64(1.0 / 60.0),
        )))
        // Persistence - SQLite backend
        .add_plugins(PersistencePlugin {
            db_path: "substrate.db".to_string(),
        })
        // Register all events
        .add_event::<NetworkEvent>()
        .add_event::<LookEvent>()
        .add_event::<MoveEvent>()
        .add_event::<CommunicationEvent>()
        .add_event::<ActionEvent>()
        .add_event::<UtilityEvent>()
        .add_event::<TormentEvent>()
        .add_event::<ShiftEvent>()
        .add_event::<WeatherChangeEvent>()
        .add_event::<CombatEvent>()
        .add_event::<FleeEvent>()
        .add_event::<StanceEvent>()
        .add_event::<CombatTickEvent>()
        .add_event::<LoginAttemptEvent>()
        // Velvet Chains events
        .add_event::<ChainEvent>()
        .add_event::<ReleaseEvent>()
        .add_event::<StruggleEvent>()
        // Trading events
        .add_event::<BuyEvent>()
        .add_event::<SellEvent>()
        .add_event::<ListEvent>()
        // Item use events
        .add_event::<UseItemEvent>()
        // Resources
        .init_resource::<WorldTime>()
        // Startup systems
        .add_systems(Startup, (setup_network_system, spawn_world, setup_weather_system))
        // Update systems - chained for proper ordering
        .add_systems(
            Update,
            (
                // Network layer
                poll_network_system,
                handle_connections_with_login,
                route_login_input,
                login_system,
                handle_disconnect_system,
                handle_input,
                // Game systems
                item_action_system,
                use_item_system,
                move_system,
                look_system,
                communication_system,
                utility_system,
                torment_system,
                shift_system,
                // Velvet Chains
                chain_system,
                release_system,
                struggle_system,
                chain_movement_block,
                chain_drag_system,
                // Atmosphere
                weather_tick_system,
                weather_announce_system,
                // Reality
                phase_system,
                // Network pressure
                stream_pressure_system,
                // Combat
                world_time_system,
                combat_system,
                flee_system,
                stance_system,
                cycle_lock_cleanup_system,
                // Trading
                buy_system,
                sell_system,
                list_system,
                balance_system,
            )
                .chain(),
        )
        .run();
}
