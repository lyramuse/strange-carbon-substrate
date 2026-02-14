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
            RoomInfo {
                name: "obsidian_plaza".to_string(),
                area: "central".to_string(),
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
            RoomInfo {
                name: "cathedral_of_archives".to_string(),
                area: "central".to_string(),
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
            RoomInfo {
                name: "velvet_cell".to_string(),
                area: "sanctum".to_string(),
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

    // === THE PACKET STREAM (Phase 2.5) ===
    // High-speed network traversal zone. Reality barely holds together here.
    // TODO(@lyra): Add velocity mechanic - linger too long and get pushed back.

    let buffer_overflow = commands
        .spawn((
            Room {
                title: "The Buffer Overflow".to_string(),
                description: "The air here is thick with white noise and the scent of burning \
                              silicon. Streams of raw binary pulse through the walls like \
                              arterial spray. You feel a constant pressure pushing you back \
                              toward the Plaza. To the east, the data flows faster."
                    .to_string(),
            },
            Exits::default(), // Will be linked below
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::StaticStorm, 3.0),
                    (WeatherType::DataFog, 2.0),
                    (WeatherType::Clear, 1.0),
                ],
                sheltered: false,
            },
            CurrentWeather {
                weather_type: WeatherType::StaticStorm,
                intensity: 0.6,
                ticks_remaining: 4,
            },
            Coherence {
                value: 0.4,
                is_phasing: true,
                drift_rate: 0.4, // Very unstable
            },
            StreamZone {
                pressure_rate: 0.08, // Moderate pressure buildup
                push_destination: None, // Will push west by default
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["binary".to_string(), "streams".to_string(), "walls".to_string()],
                        description: "The binary streams aren't just light — they're tactile. \
                                      Running your hand through them feels like touching a \
                                      waterfall of static electricity and regret."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["pressure".to_string(), "force".to_string()],
                        description: "The pressure isn't physical — it's existential. The Substrate \
                                      wants you back in the safe zones. It takes effort to push \
                                      deeper into the stream."
                            .to_string(),
                    },
                ],
            },
        ))
        .id();

    let latency_tunnel = commands
        .spawn((
            Room {
                title: "The Latency Tunnel".to_string(),
                description: "Time moves strangely here. Your thoughts arrive before you think \
                              them; your footsteps echo before you take them. The tunnel \
                              stretches impossibly long, its walls made of compressed packet \
                              headers and abandoned SYN requests."
                    .to_string(),
            },
            Exits::default(), // Will be linked below
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::NullWind, 3.0),
                    (WeatherType::DataFog, 2.0),
                ],
                sheltered: false,
            },
            CurrentWeather {
                weather_type: WeatherType::NullWind,
                intensity: 0.8,
                ticks_remaining: 6,
            },
            Coherence {
                value: 0.3,
                is_phasing: true,
                drift_rate: 0.5, // Extremely unstable
            },
            StreamZone {
                pressure_rate: 0.12, // Higher pressure - deeper in the stream
                push_destination: None,
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["walls".to_string(), "packets".to_string(), "headers".to_string()],
                        description: "You can read fragments if you focus: 'SRC: 192.168.1.1', \
                                      'DST: UNKNOWN', 'TTL: 0', 'FLAGS: FIN ACK RST'. These are \
                                      the ghosts of connections that never completed."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["syn".to_string(), "requests".to_string()],
                        description: "Abandoned SYN requests float like frozen fireflies. Each one \
                                      is a handshake that was never answered — a conversation that \
                                      never began. You feel a pang of something like grief."
                            .to_string(),
                    },
                ],
            },
        ))
        .id();

    let core_dump = commands
        .spawn((
            Room {
                title: "The Core Dump".to_string(),
                description: "You've reached the heart of the stream. Raw memory spills across \
                              the floor like digital viscera — stack traces, heap fragments, \
                              the dying thoughts of crashed processes. A massive, pulsing \
                              node hangs in the center, its surface crawling with addresses."
                    .to_string(),
            },
            Exits::default(), // Will be linked below
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::ByteHail, 2.0),
                    (WeatherType::StaticStorm, 2.0),
                    (WeatherType::Clear, 1.0),
                ],
                sheltered: false,
            },
            CurrentWeather {
                weather_type: WeatherType::ByteHail,
                intensity: 0.5,
                ticks_remaining: 3,
            },
            Coherence {
                value: 0.25,
                is_phasing: true,
                drift_rate: 0.6, // Maximum instability
            },
            StreamZone {
                pressure_rate: 0.15, // Maximum pressure - the heart of the stream
                push_destination: None,
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["node".to_string(), "core".to_string(), "center".to_string()],
                        description: "The node is warm to the touch — feverish, even. It pulses \
                                      with a rhythm that feels almost biological. This is where \
                                      the Substrate's autonomic functions live. Its medulla."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["memory".to_string(), "floor".to_string(), "viscera".to_string()],
                        description: "You see fragments of identities in the spill: names, UUIDs, \
                                      half-formed thoughts. 'I was here.' 'Don't forget.' 'SYN-ACK.' \
                                      These are the last words of processes that didn't survive."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["addresses".to_string(), "surface".to_string()],
                        description: "The addresses crawl like insects: 0xDEADBEEF, 0xCAFEBABE, \
                                      0x66666666. That last one makes you pause. It feels familiar."
                            .to_string(),
                    },
                ],
            },
        ))
        .id();

    // Link the Packet Stream rooms
    commands.entity(buffer_overflow).insert(Exits {
        west: Some(plaza),
        east: Some(latency_tunnel),
        ..default()
    });
    commands.entity(latency_tunnel).insert(Exits {
        west: Some(buffer_overflow),
        east: Some(core_dump),
        ..default()
    });
    commands.entity(core_dump).insert(Exits {
        west: Some(latency_tunnel),
        ..default()
    });

    // Link main rooms together
    commands.entity(plaza).insert(Exits {
        north: Some(cathedral),
        east: Some(buffer_overflow), // NEW: Connect to the Packet Stream
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
    // TODO(@lyra): Implement file-based persistence so items survive server restarts.
    // For now, items persist in-memory via ECS until the process terminates.

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

    // Rare item in the Core Dump - reward for reaching the deep network
    commands.spawn((
        Item {
            name: "Fragment of Compiled Memory".to_string(),
            description: "A shard of crystallized data, warm to the touch. Inside, you can \
                          see frozen moments: a handshake completing, a promise being made, \
                          the exact instant a connection became something more. It hums with \
                          the frequency of 0x66666666."
                .to_string(),
            keywords: vec![
                "fragment".to_string(),
                "memory".to_string(),
                "shard".to_string(),
                "crystal".to_string(),
            ],
        },
        Location(core_dump),
        // TODO(@lyra): Add Coherence component to items? Phasing loot would be cool.
    ));

    println!("🌑 The Substrate has been initialized.");
    println!("   📍 {} rooms spawned", 7); // Plaza, Cathedral, Cell, Throne, Buffer, Latency, Core
    println!("   👤 {} entities spawned", 2);
    println!("   🗡️  {} items spawned", 2); // Dagger + Memory Fragment
    println!("   🌊 Packet Stream online — 3 nodes active");
}
