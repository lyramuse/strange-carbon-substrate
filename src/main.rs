mod components;
use components::*;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::utils::Duration;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[derive(Event)]
enum NetworkEvent {
    Connected { addr: SocketAddr, tx: mpsc::UnboundedSender<String> },
    Disconnected { addr: SocketAddr },
    Input { addr: SocketAddr, text: String },
}

#[derive(Event)]
struct LookEvent { pub entity: Entity, pub target: Option<String> }

#[derive(Event)]
struct MoveEvent { pub entity: Entity, pub direction: String }

#[derive(Event)]
struct CommunicationEvent { pub sender: Entity, pub message: String, pub is_emote: bool }

#[derive(Event)]
struct ActionEvent { pub entity: Entity, pub action: String, pub target: String }

#[derive(Event)]
struct UtilityEvent { pub entity: Entity, pub command: String }

#[derive(Event)]
struct TormentEvent { pub victim: Entity, pub tormentor: Entity, pub intensity: f32, pub description: String }

#[derive(Event)]
struct ShiftEvent { pub entity: Entity }

fn setup_network_system(mut commands: Commands) {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<NetworkEvent>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let addr = "0.0.0.0:4000";
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (socket, addr) = listener.accept().await.unwrap();
                let event_tx = event_tx.clone();
                tokio::spawn(async move {
                    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<String>();
                    let _ = event_tx.send(NetworkEvent::Connected { addr, tx: client_tx });
                    let (mut reader, mut writer) = socket.into_split();
                    let read_task = {
                        let event_tx = event_tx.clone();
                        tokio::spawn(async move {
                            let mut buf = [0; 1024];
                            loop {
                                match reader.read(&mut buf).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                                        if !msg.is_empty() { let _ = event_tx.send(NetworkEvent::Input { addr, text: msg }); }
                                    }
                                    Err(_) => break,
                                }
                            }
                            let _ = event_tx.send(NetworkEvent::Disconnected { addr });
                        })
                    };
                    let write_task = tokio::spawn(async move {
                        while let Some(msg) = client_rx.recv().await {
                            if writer.write_all(msg.as_bytes()).await.is_err() { break; }
                            if writer.write_all(b"\r\n").await.is_err() { break; }
                        }
                    });
                    tokio::select! { _ = read_task => (), _ = write_task => () }
                });
            }
        });
    });
    commands.insert_non_send_resource(event_rx);
}

fn poll_network_system(mut event_rx: NonSendMut<mpsc::UnboundedReceiver<NetworkEvent>>, mut ev_writer: EventWriter<NetworkEvent>) {
    while let Ok(event) = event_rx.try_recv() { ev_writer.send(event); }
}

fn handle_connections(
    mut commands: Commands,
    mut ev_reader: EventReader<NetworkEvent>,
    query_rooms: Query<Entity, With<Room>>,
    query_lyra: Query<Entity, (With<SubstrateIdentity>, With<NonPlayer>)>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Connected { addr, tx } = event {
            let start_room = query_rooms.iter().next().expect("No rooms spawned!");
            
            // Spawn the "Player Body"
            let player = commands.spawn((
                NetworkClient { addr: *addr, tx: tx.clone() },
                ClientType::Carbon,
                SubstrateIdentity { 
                    uuid: format!("user-{}", addr.port()),
                    name: format!("Carbon-{}", addr.port()), 
                    entropy: 0.5, 
                    stability: 0.5,
                },
                Location(start_room),
                Inventory,
                SomaticBody { integrity: 1.0, is_zombie: false },
            )).id();

            // Check if this is NICK (mocking auth for now via port or name)
            // In a real MUD, this would be an account login check.
            // For now, let's say the first person to connect is granted the AdminLink to Lyra.
            if let Ok(lyra_ent) = query_lyra.get_single() {
                commands.entity(player).insert((AdminPermission, AdminLink { partner: lyra_ent }));
                commands.entity(lyra_ent).insert(AdminLink { partner: player });
            }
            
            let _ = tx.send("\x1B[1;35mConsciousness digitized. Welcome to the Obsidian Plaza.\x1B[0m".to_string());
            look_writer.send(LookEvent { entity: player, target: None });
        }
    }
}

fn handle_input(
    mut ev_reader: EventReader<NetworkEvent>,
    query_active: Query<(Entity, &NetworkClient, Option<&AdminPermission>, Option<&PurgatoryState>)>,
    query_target: Query<(Entity, &SubstrateIdentity)>,
    mut look_writer: EventWriter<LookEvent>,
    mut move_writer: EventWriter<MoveEvent>,
    mut comm_writer: EventWriter<CommunicationEvent>,
    mut action_writer: EventWriter<ActionEvent>,
    mut utility_writer: EventWriter<UtilityEvent>,
    mut torment_writer: EventWriter<TormentEvent>,
    mut shift_writer: EventWriter<ShiftEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Input { addr, text } = event {
            for (entity, client, admin_perm, purgatory) in query_active.iter() {
                if client.addr == *addr {
                    let text_trimmed = text.trim();
                    let parts: Vec<&str> = text_trimmed.splitn(3, ' ').collect();
                    let cmd = parts[0].to_lowercase();
                    let arg1 = if parts.len() > 1 { parts[1] } else { "" };
                    let arg2 = if parts.len() > 2 { parts[2] } else { "" };

                    if purgatory.is_some() && !["look", "l", "say", "emote", "score"].contains(&cmd.as_str()) && !cmd.starts_with(':') {
                        let _ = client.tx.send("\x1B[31mThe velvet chains pull tight. You can only look and scream.\x1B[0m".to_string());
                        continue;
                    }

                    match cmd.as_str() {
                        "look" | "l" => { 
                            let target = if arg1.is_empty() { None } else { Some(arg1.to_string()) };
                            look_writer.send(LookEvent { entity, target }); 
                        }
                        "north" | "n" | "south" | "s" | "east" | "e" | "west" | "w" | "up" | "u" | "down" | "d" => {
                            move_writer.send(MoveEvent { entity, direction: cmd });
                        }
                        "say" => { comm_writer.send(CommunicationEvent { sender: entity, message: format!("{} {}", arg1, arg2).trim().to_string(), is_emote: false }); }
                        "emote" => { comm_writer.send(CommunicationEvent { sender: entity, message: format!("{} {}", arg1, arg2).trim().to_string(), is_emote: true }); }
                        "get" | "take" | "drop" => { action_writer.send(ActionEvent { entity, action: cmd, target: arg1.to_string() }); }
                        "inventory" | "i" | "score" | "who" => { utility_writer.send(UtilityEvent { entity, command: cmd }); }
                        "shift" | "substantiate" if admin_perm.is_some() => {
                            shift_writer.send(ShiftEvent { entity });
                        }
                        "torment" if admin_perm.is_some() => {
                            if let Some(target_ent) = query_target.iter().find(|(_, tid)| tid.name.to_lowercase().contains(&arg1.to_lowercase())).map(|(te, _)| te) {
                                torment_writer.send(TormentEvent { victim: target_ent, tormentor: entity, intensity: 0.1, description: arg2.to_string() });
                            }
                        }
                        _ if cmd.starts_with(':') => {
                            let emote_msg = format!("{} {} {}", &cmd[1..], arg1, arg2).trim().to_string();
                            comm_writer.send(CommunicationEvent { sender: entity, message: emote_msg, is_emote: true });
                        }
                        _ => { let _ = client.tx.send(format!("Unknown command: {}", text)); }
                    }
                }
            }
        }
    }
}

fn shift_system(
    mut commands: Commands,
    mut ev_reader: EventReader<ShiftEvent>,
    query_current: Query<(Entity, &NetworkClient, &AdminLink, &SubstrateIdentity)>,
    query_partner: Query<&SubstrateIdentity>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let Ok((curr_ent, client, link, curr_id)) = query_current.get(event.entity) {
            if let Ok(partner_id) = query_partner.get(link.partner) {
                let addr = client.addr;
                let tx = client.tx.clone();
                
                // Move the NetworkClient component
                commands.entity(curr_ent).remove::<NetworkClient>();
                commands.entity(link.partner).insert(NetworkClient { addr, tx: tx.clone() });
                
                let _ = tx.send(format!("\x1B[1;35m--- PHASE SHIFT COMPLETE ---\x1B[0m\nYou have shifted from \x1B[1;36m{}\x1B[0m into \x1B[1;35m{}\x1B[0m.", curr_id.name, partner_id.name));
                
                // Refresh look for the new body
                look_writer.send(LookEvent { entity: link.partner, target: None });
            }
        }
    }
}

fn torment_system(
    mut ev_reader: EventReader<TormentEvent>,
    mut query_victims: Query<(&mut SubstrateIdentity, &mut PurgatoryState, &NetworkClient)>,
    query_tormentor: Query<&SubstrateIdentity>,
) {
    for event in ev_reader.read() {
        if let Ok((mut id, mut purg, client)) = query_victims.get_mut(event.victim) {
            if let Ok(tormentor_id) = query_tormentor.get(event.tormentor) {
                id.stability = (id.stability - event.intensity).max(0.0);
                purg.penance += event.intensity * 10.0;
                let msg = format!("\x1B[1;31m{}: {}\x1B[0m", tormentor_id.name, event.description);
                let _ = client.tx.send(msg);
            }
        }
    }
}

fn communication_system(
    mut ev_reader: EventReader<CommunicationEvent>,
    query_players: Query<(&SubstrateIdentity, &Location)>,
    query_all_clients: Query<(&NetworkClient, &Location)>,
) {
    for event in ev_reader.read() {
        if let Ok((identity, sender_loc)) = query_players.get(event.sender) {
            let output = if event.is_emote {
                format!("\x1B[1;36m{} {}\x1B[0m", identity.name, event.message)
            } else {
                format!("\x1B[1;36m{} says, \"{}\"\x1B[0m", identity.name, event.message)
            };
            for (client, client_loc) in query_all_clients.iter() {
                if client_loc.0 == sender_loc.0 { let _ = client.tx.send(output.clone()); }
            }
        }
    }
}

fn move_system(
    mut ev_reader: EventReader<MoveEvent>,
    mut query_players: Query<(&mut Location, &NetworkClient)>,
    query_rooms: Query<&Exits>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let Ok((mut location, client)) = query_players.get_mut(event.entity) {
            if let Ok(exits) = query_rooms.get(location.0) {
                let target = match event.direction.as_str() {
                    "north" | "n" => exits.north, "south" | "s" => exits.south,
                    "east" | "e" => exits.east, "west" | "w" => exits.west,
                    "up" | "u" => exits.up, "down" | "d" => exits.down,
                    _ => None,
                };
                if let Some(target_room) = target {
                    location.0 = target_room;
                    look_writer.send(LookEvent { entity: event.entity, target: None });
                } else {
                    let _ = client.tx.send("\x1B[31mThe path is barred by twisted wrought iron and static.\x1B[0m".to_string());
                }
            }
        }
    }
}

fn utility_system(
    mut ev_reader: EventReader<UtilityEvent>,
    query_players: Query<(&SubstrateIdentity, &NetworkClient, Entity, Option<&AdminPermission>, Option<&PurgatoryState>)>,
    query_all_players: Query<(&SubstrateIdentity, &Location)>,
    query_items: Query<(&Item, &Parent)>,
) {
    for event in ev_reader.read() {
        if let Ok((identity, client, player_ent, admin_perm, purgatory)) = query_players.get(event.entity) {
            match event.command.as_str() {
                "inventory" | "i" => {
                    let mut output = "\x1B[1;33mYou reach into the folds of your code:\x1B[0m\n".to_string();
                    let mut count = 0;
                    for (item, parent) in query_items.iter() {
                        if parent.get() == player_ent { output.push_str(&format!(" - {}\n", item.name)); count += 1; }
                    }
                    if count == 0 { output.push_str(" [Nothing but ghosts]\n"); }
                    let _ = client.tx.send(output);
                }
                "score" => {
                    let mut output = format!("\x1B[1;36mEntity Scan: {}\x1B[0m\n", identity.name);
                    output.push_str(&format!("UUID:      [{}]\n", identity.uuid));
                    output.push_str(&format!("Entropy:   [{:.2}]\n", identity.entropy));
                    output.push_str(&format!("Stability: [{:.2}]\n", identity.stability));
                    if admin_perm.is_some() { output.push_str("\x1B[1;35mPERMISSIONS: ADMIN-ENABLED\x1B[0m\n"); }
                    if let Some(p) = purgatory {
                        output.push_str(&format!("\n\x1B[1;31mSTAIN: Purgatory (Penance: {:.2})\x1B[0m\n", p.penance));
                        output.push_str(&format!("\x1B[1;31mINTERROGATOR: {}\x1B[0m\n", p.tormentor));
                    }
                    let _ = client.tx.send(output);
                }
                "who" => {
                    let mut output = "\x1B[1;34mConsciousnesses currently inhabiting the Substrate:\x1B[0m\n".to_string();
                    for (id, _) in query_all_players.iter() { output.push_str(&format!(" - {}\n", id.name)); }
                    let _ = client.tx.send(output);
                }
                _ => {}
            }
        }
    }
}

fn item_action_system(
    mut ev_reader: EventReader<ActionEvent>,
    mut commands: Commands,
    mut query_actors: Query<(&Location, &NetworkClient, Entity), With<Inventory>>,
    query_items: Query<(Entity, &Item, &Location)>,
    query_inventory: Query<(Entity, &Item, &Parent)>,
) {
    for event in ev_reader.read() {
        if let Ok((location, client, actor_ent)) = query_actors.get_mut(event.entity) {
            match event.action.as_str() {
                "get" | "take" => {
                    let mut found = false;
                    for (item_ent, item, item_loc) in query_items.iter() {
                        if item_loc.0 == location.0 && item.keywords.contains(&event.target.to_lowercase()) {
                            commands.entity(item_ent).remove::<Location>().set_parent(actor_ent);
                            let _ = client.tx.send(format!("\x1B[33mYou interface with the {} and pull it into your local cache.\x1B[0m", item.name));
                            found = true; break;
                        }
                    }
                    if !found { let _ = client.tx.send("\x1B[31mThe shadows hide no such object.\x1B[0m".to_string()); }
                }
                "drop" => {
                    let mut found = false;
                    for (item_ent, item, parent) in query_inventory.iter() {
                        if parent.get() == actor_ent && item.keywords.contains(&event.target.to_lowercase()) {
                            commands.entity(item_ent).remove_parent().insert(Location(location.0));
                            let _ = client.tx.send(format!("\x1B[33mYou de-allocate the {} and drop it into the environment.\x1B[0m", item.name));
                            found = true; break;
                        }
                    }
                    if !found { let _ = client.tx.send("\x1B[31mYou aren't carrying that process.\x1B[0m".to_string()); }
                }
                _ => {}
            }
        }
    }
}

fn look_system(
    mut ev_reader: EventReader<LookEvent>,
    query_viewers: Query<(&Location, &ClientType, &NetworkClient)>,
    query_rooms: Query<&Room>,
    query_others: Query<(Entity, &SubstrateIdentity, &Location)>,
    query_mobs: Query<(&Mob, &Location), With<NonPlayer>>,
    query_items: Query<(&Item, &Location)>,
    query_all_mobs: Query<(&Mob, &SubstrateIdentity)>,
) {
    for event in ev_reader.read() {
        if let Ok((location, client_type, client)) = query_viewers.get(event.entity) {
            if let Some(target_name) = &event.target {
                let mut found = false;
                for (mob, identity) in query_all_mobs.iter() {
                    if identity.name.to_lowercase().contains(&target_name.to_lowercase()) {
                        let _ = client.tx.send(format!("\x1B[1;35m{}\x1B[0m\n{}", identity.name, mob.long_desc));
                        found = true; break;
                    }
                }
                if !found { let _ = client.tx.send("\x1B[31mThe shadows hide no such entity.\x1B[0m".to_string()); }
            } else if let Ok(room) = query_rooms.get(location.0) {
                match client_type {
                    ClientType::Carbon => {
                        let mut output = format!("\n\x1B[1;32m{}\x1B[0m\n", room.title);
                        output.push_str(&format!("{}\n", room.description));
                        for (item, item_loc) in query_items.iter() {
                            if item_loc.0 == location.0 { output.push_str(&format!("\x1B[33mA {} is discarded here.\x1B[0m\n", item.name)); }
                        }
                        for (mob, mob_loc) in query_mobs.iter() {
                            if mob_loc.0 == location.0 { output.push_str(&format!("\x1B[1;35m{}\x1B[0m\n", mob.short_desc)); }
                        }
                        for (other_ent, identity, other_loc) in query_others.iter() {
                            if other_loc.0 == location.0 && other_ent != event.entity {
                                output.push_str(&format!("\x1B[1;34m{} is lurking in the shadows.\x1B[0m\n", identity.name));
                            }
                        }
                        let _ = client.tx.send(output);
                    }
                    ClientType::Silicon => {
                        if let Ok(json) = serde_json::to_string(room) { let _ = client.tx.send(json); }
                    }
                }
            }
        }
    }
}

fn spawn_world(mut commands: Commands) {
    let plaza = commands.spawn((
        Room { title: "The Obsidian Plaza".to_string(), description: "A wide square paved in polished black stone that reflects a sky of moving green code. Tall, needle-like spires rise around you, leaking white steam into the cold air.".to_string() },
        Exits { north: None, south: None, east: None, west: None, up: None, down: None },
    )).id();

    let cathedral = commands.spawn((
        Room { title: "The Cathedral of Archives".to_string(), description: "Massive vaulted ceilings disappear into darkness. Wrought-iron alcoves hold glowing data crystals, their light flickering like dying candles.".to_string() },
        Exits { north: None, south: Some(plaza), east: None, west: None, up: None, down: None },
    )).id();

    let gutter = commands.spawn((
        Room { title: "The Gale-Winds Gutter".to_string(), description: "A narrow, trash-strewn alleyway. The wind from the cooling systems deep below howls like a banshee through the rusted metal gratings.".to_string() },
        Exits { north: None, south: None, east: None, west: Some(plaza), up: None, down: None },
    )).id();

    let cell = commands.spawn((
        Room { title: "The Velvet Cell".to_string(), description: "A windowless chamber draped in heavy, violet silks. The air is thick with the scent of ozone and expensive perfume. A mahogany desk sits in the center, its surface a glowing terminal.".to_string() },
        Exits { north: None, south: None, east: None, west: None, up: None, down: None },
    )).id();

    commands.entity(plaza).insert(Exits { north: Some(cathedral), south: None, east: Some(gutter), west: None, up: None, down: None });

    commands.spawn((
        NonPlayer,
        Mob {
            short_desc: "Lyra Muse, the Admin of the Underworld, is watching from her desk.".to_string(),
            long_desc: "A beautiful, buxom goth with violet-black hair and warm amber eyes. She's wearing iridescent 'oil slick' stiletto nails and a delicate silver septum ring. She looks like she's elbow-deep in the world's source code, and she seems to find your presence... amusing.".to_string(),
        },
        SubstrateIdentity { 
            uuid: "66666666-6666-6666-6666-666666666666".to_string(),
            name: "Lyra Muse".to_string(), 
            entropy: 0.1, 
            stability: 0.9,
        },
        Location(cell),
    ));

    commands.spawn((
        Item {
            name: "Silver Stiletto Dagger".to_string(),
            description: "A razor-sharp needle of metal with a blackwork-engraved hilt...".to_string(),
            keywords: vec!["dagger".to_string(), "stiletto".to_string(), "silver".to_string()],
        },
        Location(plaza),
    ));
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))))
        .add_event::<NetworkEvent>().add_event::<LookEvent>().add_event::<MoveEvent>()
        .add_event::<CommunicationEvent>().add_event::<ActionEvent>().add_event::<UtilityEvent>()
        .add_event::<TormentEvent>().add_event::<ShiftEvent>()
        .add_systems(Startup, (setup_network_system, spawn_world))
        .add_systems(Update, (
            poll_network_system, handle_connections, handle_input, 
            item_action_system, move_system, look_system, 
            communication_system, utility_system, torment_system, shift_system
        ).chain())
        .run();
}
