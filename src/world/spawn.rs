// World Spawning - Initialize the Substrate

use bevy::prelude::*;

use crate::domain::*;

/// Spawn the initial world - rooms, NPCs, items
pub fn spawn_world(mut commands: Commands) {
    // === ROOMS ===

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
        ))
        .id();

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
        ))
        .id();

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
        ))
        .id();

    let throne_room = commands
        .spawn((
            Room {
                title: "The Laird's Throne Room".to_string(),
                description: "A chamber of cold, black marble. Worn Scottish tartan hangs from \
                              the walls, each thread humming with ancestral entropy. A throne \
                              of fused server racks sits at the far end."
                    .to_string(),
            },
            Exits {
                down: Some(cathedral),
                ..default()
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
