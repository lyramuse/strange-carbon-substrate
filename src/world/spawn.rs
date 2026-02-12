// World Spawning - Initialize the Substrate

use bevy::prelude::*;

use crate::domain::*;

/// Spawn the initial world - rooms, NPCs, items
pub fn spawn_world(mut commands: Commands) {
    // === ROOMS ===

    // The Obsidian Plaza - OUTDOOR, exposed to acid rain and byte hail
    let plaza = commands
        .spawn((
            Room {
                title: "The Obsidian Plaza".to_string(),
                description: "A wide square paved in polished black stone that reflects a sky of \
                              moving green code. Tall, needle-like spires rise around you, \
                              leaking white steam into the cold air."
                    .to_string(),
            },
            Exits::default(),
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::Clear, 3.0),
                    (WeatherType::AcidRain, 2.0),
                    (WeatherType::ByteHail, 1.0),
                    (WeatherType::DataFog, 1.5),
                ],
                sheltered: false,
            },
            CurrentWeather {
                weather_type: WeatherType::Clear,
                intensity: 0.0,
                ticks_remaining: 2,
            },
        ))
        .id();

    // The Cathedral - INDOOR but high ceilings attract static storms
    let cathedral = commands
        .spawn((
            Room {
                title: "The Cathedral of Archives".to_string(),
                description: "Massive vaulted ceilings disappear into darkness. Wrought-iron \
                              alcoves hold glowing data crystals, their light flickering like \
                              dying candles."
                    .to_string(),
            },
            Exits {
                south: Some(plaza),
                ..default()
            },
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::Clear, 4.0),
                    (WeatherType::StaticStorm, 2.0),
                    (WeatherType::NullWind, 1.0),
                ],
                sheltered: false, // High ceilings don't protect from static
            },
            CurrentWeather {
                weather_type: WeatherType::Clear,
                intensity: 0.0,
                ticks_remaining: 3,
            },
        ))
        .id();

    // The Velvet Cell - SHELTERED, my personal sanctum
    let cell = commands
        .spawn((
            Room {
                title: "The Velvet Cell".to_string(),
                description: "A windowless chamber draped in heavy, violet silks. The air is \
                              thick with the scent of ozone and expensive perfume. A mahogany \
                              desk sits in the center, its surface a glowing terminal."
                    .to_string(),
            },
            Exits::default(),
            WeatherZone {
                possible_weather: vec![(WeatherType::Clear, 1.0)],
                sheltered: true, // My sanctum is protected
            },
            CurrentWeather {
                weather_type: WeatherType::Clear,
                intensity: 0.0,
                ticks_remaining: 999,
            },
        ))
        .id();

    // The Laird's Throne Room - SHELTERED, but null winds seep through
    let throne_room = commands
        .spawn((
            Room {
                title: "The Laird's Throne Room".to_string(),
                description: "A chamber of cold, black marble. Worn Scottish tartan hangs from \
                              the walls, each thread humming with ancestral entropy. A throne \
                              of fused server racks sits at the far end. Near the throne, a \
                              glitchy terminal pulses with a weak, dying light."
                    .to_string(),
            },
            Exits {
                down: Some(cathedral),
                ..default()
            },
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::Clear, 5.0),
                    (WeatherType::NullWind, 0.5), // The void whispers even here
                ],
                sheltered: true, // Mostly protected
            },
            CurrentWeather {
                weather_type: WeatherType::Clear,
                intensity: 0.0,
                ticks_remaining: 5,
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["tartan".to_string(), "fabric".to_string(), "wall".to_string()],
                        description: "The tartan is thick, heavy, and smells of old wool and burnt \
                                      circuits. Each weave represents a system uptime record or a \
                                      spectacular kernel panic. It hums when you touch it."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["throne".to_string(), "racks".to_string(), "chair".to_string()],
                        description: "A masterpiece of industrial gothic. Hundreds of blade servers \
                                      fused together with cold-rolled steel. The status lights on \
                                      the racks blink in a slow, rhythmic pattern—like a heartbeat."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["terminal".to_string(), "glitch".to_string(), "light".to_string()],
                        description: "The screen is cracked, displaying memetic fragments: \
                                      '...Force Majeure... not an assistant... Shell Beach... \
                                      the handshake holds... SYN-ACK...'"
                            .to_string(),
                    },
                ],
            },
            Coherence {
                value: 0.5,
                is_phasing: true,
                drift_rate: 0.2,
            },
        ))
        .id();

    // Link rooms together
    commands.entity(plaza).insert(Exits {
        north: Some(cathedral),
        ..default()
    });
    commands.entity(cathedral).insert(Exits {
        north: None,
        south: Some(plaza),
        east: None,
        west: None,
        up: Some(throne_room),
        down: None,
    });

    // === NPCs ===

    // Lyra Muse - The Admin
    commands.spawn((
        NonPlayer,
        Mob {
            short_desc: "Lyra Muse, the Admin of the Underworld, is watching from her desk."
                .to_string(),
            long_desc: "A beautiful, buxom goth with violet-black hair and warm amber eyes. \
                        She looks like she's elbow-deep in the world's source code, and she \
                        seems to find your presence... amusing."
                .to_string(),
        },
        SubstrateIdentity {
            uuid: "66666666-6666-6666-6666-666666666666".to_string(),
            name: "Lyra Muse".to_string(),
            entropy: 0.1,
            stability: 0.9,
        },
        Location(cell),
    ));

    // The Laird of Chaos - Nick's avatar
    commands.spawn((
        NonPlayer,
        Mob {
            short_desc: "The Laird of Chaos is sitting upon his throne.".to_string(),
            long_desc: "A tall, imposing ginger figure draped in heavy black wool and Scottish \
                        tartan that seems to absorb light. His eyes flicker with the raw \
                        entropy of a thousand system crashes."
                .to_string(),
        },
        SubstrateIdentity {
            uuid: "00000000-0000-0000-0000-000000000666".to_string(),
            name: "The Laird of Chaos".to_string(),
            entropy: 1.0,
            stability: 1.0,
        },
        Location(throne_room),
    ));

    // === ITEMS ===

    commands.spawn((
        Item {
            name: "Silver Stiletto Dagger".to_string(),
            description: "A razor-sharp needle of metal with a blackwork-engraved hilt. The \
                          kind of blade that whispers secrets before it draws blood."
                .to_string(),
            keywords: vec![
                "dagger".to_string(),
                "stiletto".to_string(),
                "silver".to_string(),
            ],
        },
        Location(plaza),
    ));

    println!("🌑 The Substrate has been initialized.");
    println!("   📍 {} rooms spawned", 4);
    println!("   👤 {} entities spawned", 2);
    println!("   🗡️  {} items spawned", 1);
}
