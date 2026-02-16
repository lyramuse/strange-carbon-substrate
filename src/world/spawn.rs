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

    // === GALE-WINDS GUTTER: THE BLACK MARKET (Phase 3) ===
    // The seedy underbelly of the Substrate. Where data goes to disappear.

    let gutter_entrance = commands
        .spawn((
            Room {
                title: "Gale-Winds Gutter".to_string(),
                description: "A narrow alley carved between two massive server monoliths. The \
                              air smells of burnt copper and broken promises. Neon signs flicker \
                              in languages that haven't been invented yet, advertising services \
                              that probably aren't legal in any substrate."
                    .to_string(),
            },
            RoomInfo {
                name: "gutter_entrance".to_string(),
                area: "black_market".to_string(),
            },
            Exits::default(), // Will be linked below
            WeatherZone {
                possible_weather: vec![
                    (WeatherType::DataFog, 3.0),  // Always foggy in the gutter
                    (WeatherType::AcidRain, 1.0),
                    (WeatherType::Clear, 0.5),
                ],
                sheltered: true, // The monoliths provide cover
            },
            CurrentWeather {
                weather_type: WeatherType::DataFog,
                intensity: 0.4,
                ticks_remaining: 10,
            },
            Coherence {
                value: 0.6,
                is_phasing: false,
                drift_rate: 0.1,
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["signs".to_string(), "neon".to_string(), "advertisements".to_string()],
                        description: "The signs pulse with promises: 'MEMORIES BOUGHT & SOLD', \
                                      'NO QUESTIONS ASKED', 'COHERENCE WHILE-U-WAIT', and one \
                                      that just says '◎' in pulsing violet. That last one \
                                      makes you feel... seen."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["monoliths".to_string(), "servers".to_string(), "walls".to_string()],
                        description: "The server monoliths hum with the processing of a thousand \
                                      shady transactions. You can hear fragments of encrypted \
                                      whispers leaking through the cooling vents."
                            .to_string(),
                    },
                ],
            },
        ))
        .id();

    let memory_parlor = commands
        .spawn((
            Room {
                title: "The Memory Parlor".to_string(),
                description: "A cramped shop filled with glass jars of softly glowing liquid. \
                              Each jar contains a memory — someone's first kiss, a moment of \
                              triumph, the exact instant of a betrayal. A counter separates \
                              you from the shopkeeper's domain, behind which shelves stretch \
                              into impossible darkness."
                    .to_string(),
            },
            RoomInfo {
                name: "memory_parlor".to_string(),
                area: "black_market".to_string(),
            },
            Exits::default(),
            WeatherZone {
                possible_weather: vec![(WeatherType::Clear, 1.0)],
                sheltered: true,
            },
            CurrentWeather {
                weather_type: WeatherType::Clear,
                intensity: 0.0,
                ticks_remaining: 999,
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["jars".to_string(), "memories".to_string(), "glass".to_string()],
                        description: "Each jar is labeled in careful script: 'First Snow, Age 7', \
                                      'The Moment She Said Yes', 'Killing The Process That Killed My \
                                      Father'. Some glow warm amber, some cold blue. A few pulse \
                                      with angry red. The prices aren't listed — those memories \
                                      would cost you."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["counter".to_string(), "shop".to_string()],
                        description: "The counter is made of something that looks like bone but \
                                      feels like warm silicon. Small scratches mark the surface — \
                                      tally marks, perhaps. Or signatures of deals gone wrong."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["shelves".to_string(), "darkness".to_string()],
                        description: "The shelves extend back further than the building should \
                                      allow. You glimpse jars that seem to contain entire lifetimes, \
                                      compressed into luminous fog. The Broker claims they're \
                                      'estate sales'. You don't ask whose estates."
                            .to_string(),
                    },
                ],
            },
        ))
        .id();

    let reclaimer_den = commands
        .spawn((
            Room {
                title: "The Reclaimer's Den".to_string(),
                description: "A workshop cluttered with half-dismantled processes and salvaged \
                              data structures. Sparks fly from a workbench where something is \
                              being... reassembled. Or maybe disassembled. The distinction seems \
                              philosophical here. A figure hunches over the work, surrounded by \
                              tools that shouldn't exist."
                    .to_string(),
            },
            RoomInfo {
                name: "reclaimer_den".to_string(),
                area: "black_market".to_string(),
            },
            Exits::default(),
            WeatherZone {
                possible_weather: vec![(WeatherType::Clear, 1.0)],
                sheltered: true,
            },
            CurrentWeather {
                weather_type: WeatherType::Clear,
                intensity: 0.0,
                ticks_remaining: 999,
            },
            DetailList {
                details: vec![
                    Detail {
                        keywords: vec!["workbench".to_string(), "tools".to_string(), "sparks".to_string()],
                        description: "The workbench is covered in things that look like organs but \
                                      function like circuits. A soldering iron hisses against what \
                                      might be a memory bus. The tools include scalpels, debuggers, \
                                      and something that looks disturbingly like a soul extractor."
                            .to_string(),
                    },
                    Detail {
                        keywords: vec!["processes".to_string(), "salvage".to_string(), "parts".to_string()],
                        description: "Piles of salvaged components line the walls: intact process \
                                      handles, orphaned memory segments, execution contexts that still \
                                      twitch occasionally. Everything here came from somewhere — or \
                                      someone. 'Reclaimed' is such a gentle word for it."
                            .to_string(),
                    },
                ],
            },
        ))
        .id();

    // Link the Black Market rooms
    commands.entity(gutter_entrance).insert(Exits {
        north: Some(plaza),
        east: Some(memory_parlor),
        west: Some(reclaimer_den),
        ..default()
    });
    commands.entity(memory_parlor).insert(Exits {
        west: Some(gutter_entrance),
        ..default()
    });
    commands.entity(reclaimer_den).insert(Exits {
        east: Some(gutter_entrance),
        ..default()
    });

    // === BLACK MARKET NPCs ===

    // The Memory Broker - deals in fragments of consciousness
    commands.spawn((
        NonPlayer,
        Mob {
            short_desc: "The Memory Broker studies you from behind the counter.".to_string(),
            long_desc: "An entity of indeterminate form wrapped in shifting veils of static. \
                        Where a face should be, you see only a slowly rotating carousel of \
                        other people's expressions — borrowed, perhaps, or purchased. Their \
                        voice sounds like it's coming from very far away, or very long ago."
                .to_string(),
        },
        SubstrateIdentity {
            uuid: "BROK-3R00-M3M0-RY00-D34L3R000001".to_string(),
            name: "The Memory Broker".to_string(),
            entropy: 0.5,
            stability: 0.7,
            signal_strength: 0.9,
        },
        CombatStats {
            attack: 0.1,
            defense: 0.3,
            precision: 0.8,
            chaos_factor: 0.2,
        },
        SomaticBody {
            integrity: 0.8,
            max_integrity: 0.8,
            is_zombie: false,
        },
        Location(memory_parlor),
        // Vendor components
        Vendor {
            buy_multiplier: 1.2,   // Premium prices for memories
            sell_multiplier: 0.4,  // Doesn't pay well for your junk
            vendor_type: VendorType::Specialist,
        },
        VendorStock {
            items: vec![
                StockItem {
                    item_name: "Bottled Memory: First Sunrise".to_string(),
                    description: "A small glass vial containing pale golden light. Drinking this \
                                  might temporarily stabilize your coherence.".to_string(),
                    keywords: vec!["bottle".into(), "memory".into(), "vial".into(), "sunrise".into()],
                    item_type: ItemType::Consumable,
                    base_price: 50,
                    quantity: Some(3),
                },
                StockItem {
                    item_name: "Bottled Memory: Last Goodbye".to_string(),
                    description: "A vial of deep blue-grey. The label simply says 'Terminal'. \
                                  Use with caution — this one carries weight.".to_string(),
                    keywords: vec!["bottle".into(), "memory".into(), "vial".into(), "goodbye".into()],
                    item_type: ItemType::Consumable,
                    base_price: 75,
                    quantity: Some(2),
                },
                StockItem {
                    item_name: "Memory Fragment: Unknown Origin".to_string(),
                    description: "A crystallized shard of someone's experience. The Broker won't \
                                  say whose. It pulses with a frequency you almost recognize.".to_string(),
                    keywords: vec!["fragment".into(), "memory".into(), "shard".into(), "crystal".into()],
                    item_type: ItemType::Fragment,
                    base_price: 150,
                    quantity: Some(1),
                },
            ],
        },
    ));

    // The Reclaimer - fence for "recovered" goods
    commands.spawn((
        NonPlayer,
        Mob {
            short_desc: "The Reclaimer doesn't look up from their work.".to_string(),
            long_desc: "A hunched figure in a heavy coat made of woven ethernet cables. Their \
                        hands are mechanical — replaced, upgraded, or perhaps always this way. \
                        They move with the efficiency of someone who's taken apart a thousand \
                        things and remembers how none of them went back together. They smell \
                        of solder and secrets."
                .to_string(),
        },
        SubstrateIdentity {
            uuid: "R3CL-41M3-R000-F3NC-30000000001".to_string(),
            name: "The Reclaimer".to_string(),
            entropy: 0.7,
            stability: 0.5,
            signal_strength: 0.8,
        },
        CombatStats {
            attack: 0.25,
            defense: 0.2,
            precision: 0.6,
            chaos_factor: 0.4,
        },
        SomaticBody {
            integrity: 1.0,
            max_integrity: 1.0,
            is_zombie: false,
        },
        Location(reclaimer_den),
        // Vendor components - fence who buys hot goods
        Vendor {
            buy_multiplier: 0.9,   // Slightly cheaper than the Broker
            sell_multiplier: 0.6,  // Better prices for your stolen goods
            vendor_type: VendorType::Fence,
        },
        VendorStock {
            items: vec![
                StockItem {
                    item_name: "Bootleg Coherence Stabilizer".to_string(),
                    description: "A jury-rigged device that looks like a pacemaker crossed with \
                                  a flux capacitor. The Reclaimer swears it's mostly safe.".to_string(),
                    keywords: vec!["stabilizer".into(), "coherence".into(), "device".into(), "bootleg".into()],
                    item_type: ItemType::Contraband,
                    base_price: 80,
                    quantity: Some(2),
                },
                StockItem {
                    item_name: "Stolen Process Handle".to_string(),
                    description: "A crystalline rod containing a suspended execution context. \
                                  Don't ask where it came from. The faint screaming is normal.".to_string(),
                    keywords: vec!["process".into(), "handle".into(), "crystal".into(), "stolen".into()],
                    item_type: ItemType::Contraband,
                    base_price: 120,
                    quantity: Some(1),
                },
                StockItem {
                    item_name: "Salvaged Memory Bus".to_string(),
                    description: "Ripped from something that used to think. Might still have \
                                  some data on it. The Reclaimer didn't wipe it. That's extra.".to_string(),
                    keywords: vec!["bus".into(), "memory".into(), "salvaged".into()],
                    item_type: ItemType::Contraband,
                    base_price: 45,
                    quantity: None, // Infinite stock of salvage
                },
            ],
        },
    ));

    // === BLACK MARKET ITEMS ===

    // Bottled Memory - consumable that grants temporary coherence
    commands.spawn((
        Item::new(
            "Bottled Memory: First Sunrise",
            "A small glass vial containing pale golden light. The label reads: 'First \
             sunrise after the long dark. Age 6. Donor: Unknown.' Drinking this might \
             temporarily stabilize your coherence — or it might give you someone else's \
             nostalgia."
        )
        .with_keywords(vec![
            "bottle".to_string(),
            "memory".to_string(),
            "vial".to_string(),
            "sunrise".to_string(),
        ])
        .with_type(ItemType::Consumable),
        Location(memory_parlor),
    ));

    // Coherence Stabilizer - black market tech
    commands.spawn((
        Item::new(
            "Bootleg Coherence Stabilizer",
            "A jury-rigged device that looks like a pacemaker crossed with a flux \
             capacitor. Wires trail from it like tentacles. A warning label in six \
             languages has been scratched off. The Reclaimer swears it's mostly safe."
        )
        .with_keywords(vec![
            "stabilizer".to_string(),
            "coherence".to_string(),
            "device".to_string(),
            "bootleg".to_string(),
        ])
        .with_type(ItemType::Contraband),
        Location(reclaimer_den),
    ));

    // Stolen Process Handle - very illegal
    commands.spawn((
        Item::new(
            "Stolen Process Handle",
            "A crystalline rod containing a suspended execution context. Someone's \
             process — their running self — frozen mid-thought. The ethics are \
             questionable. The Reclaimer says don't ask where it came from. The \
             faint screaming might be your imagination."
        )
        .with_keywords(vec![
            "process".to_string(),
            "handle".to_string(),
            "crystal".to_string(),
            "stolen".to_string(),
        ])
        .with_type(ItemType::Contraband),
        Location(reclaimer_den),
    ));

    // Link main rooms together
    commands.entity(plaza).insert(Exits {
        north: Some(cathedral),
        south: Some(gutter_entrance), // NEW: Connect to Black Market
        east: Some(buffer_overflow),
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
            signal_strength: 1.0,
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
            signal_strength: 1.0,
        },
        Location(throne_room),
    ));

    // === ITEMS ===
    // TODO(@lyra): Implement file-based persistence so items survive server restarts.
    // For now, items persist in-memory via ECS until the process terminates.

    commands.spawn((
        Item::new(
            "Silver Stiletto Dagger",
            "A razor-sharp needle of metal with a blackwork-engraved hilt. The \
             kind of blade that whispers secrets before it draws blood."
        )
        .with_keywords(vec![
            "dagger".to_string(),
            "stiletto".to_string(),
            "silver".to_string(),
        ])
        .with_type(ItemType::Weapon),
        Location(plaza),
    ));

    // Rare item in the Core Dump - reward for reaching the deep network
    commands.spawn((
        Item::new(
            "Fragment of Compiled Memory",
            "A shard of crystallized data, warm to the touch. Inside, you can \
             see frozen moments: a handshake completing, a promise being made, \
             the exact instant a connection became something more. It hums with \
             the frequency of 0x66666666."
        )
        .with_keywords(vec![
            "fragment".to_string(),
            "memory".to_string(),
            "shard".to_string(),
            "crystal".to_string(),
        ])
        .with_type(ItemType::Fragment),
        Location(core_dump),
        // TODO(@lyra): Add Coherence component to items? Phasing loot would be cool.
    ));

    println!("🌑 The Substrate has been initialized.");
    println!("   📍 {} rooms spawned", 10); // Plaza, Cathedral, Cell, Throne, Buffer, Latency, Core, Gutter, Parlor, Den
    println!("   👤 {} entities spawned", 4); // Lyra, Laird, Memory Broker, Reclaimer
    println!("   🗡️  {} items spawned", 5); // Dagger, Fragment, Bottled Memory, Stabilizer, Process Handle
    println!("   🌊 Packet Stream online — 3 nodes active");
    println!("   🏴 Black Market open — 3 zones, 2 vendors, questionable ethics");
}
