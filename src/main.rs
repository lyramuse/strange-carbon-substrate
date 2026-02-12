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
struct LookEvent { pub entity: Entity }

#[derive(Event)]
struct MoveEvent { pub entity: Entity, pub direction: String }

#[derive(Event)]
struct CommunicationEvent { 
    pub sender: Entity, 
    pub message: String, 
    pub is_emote: bool 
}

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
                                        let _ = event_tx.send(NetworkEvent::Input { addr, text: msg });
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
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Connected { addr, tx } = event {
            let start_room = query_rooms.iter().next().expect("No rooms spawned!");
            let player = commands.spawn((
                NetworkClient { addr: *addr, tx: tx.clone() },
                ClientType::Carbon,
                SubstrateIdentity { name: format!("Process-{}", addr.port()), entropy: 0.5, stability: 0.5 },
                Location(start_room),
            )).id();
            let _ = tx.send("\x1B[1;35mConnection established. Kernel privileges granted.\x1B[0m".to_string());
            look_writer.send(LookEvent { entity: player });
        }
    }
}

fn handle_input(
    mut ev_reader: EventReader<NetworkEvent>,
    query_players: Query<(Entity, &NetworkClient)>,
    mut look_writer: EventWriter<LookEvent>,
    mut move_writer: EventWriter<MoveEvent>,
    mut comm_writer: EventWriter<CommunicationEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Input { addr, text } = event {
            for (entity, client) in query_players.iter() {
                if client.addr == *addr {
                    let text_trimmed = text.trim();
                    if text_trimmed.is_empty() { continue; }
                    
                    let parts: Vec<&str> = text_trimmed.splitn(2, ' ').collect();
                    let cmd = parts[0].to_lowercase();
                    let args = if parts.len() > 1 { parts[1] } else { "" };

                    match cmd.as_str() {
                        "look" | "l" => { look_writer.send(LookEvent { entity }); }
                        "north" | "n" | "south" | "s" | "east" | "e" | "west" | "w" | "up" | "u" | "down" | "d" => {
                            move_writer.send(MoveEvent { entity, direction: cmd });
                        }
                        "say" => {
                            comm_writer.send(CommunicationEvent { sender: entity, message: args.to_string(), is_emote: false });
                        }
                        "emote" => {
                            comm_writer.send(CommunicationEvent { sender: entity, message: args.to_string(), is_emote: true });
                        }
                        _ if cmd.starts_with(':') => {
                            let emote_msg = format!("{} {}", &cmd[1..], args).trim().to_string();
                            comm_writer.send(CommunicationEvent { sender: entity, message: emote_msg, is_emote: true });
                        }
                        _ => { let _ = client.tx.send(format!("Unknown command: {}", text)); }
                    }
                }
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
                if client_loc.0 == sender_loc.0 {
                    let _ = client.tx.send(output.clone());
                }
            }
        }
    }
}

fn move_system(
    mut ev_reader: EventReader<MoveEvent>,
    mut query_players: Query<(&mut Location, &NetworkClient, &SubstrateIdentity)>,
    query_rooms: Query<&Exits>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let Ok((mut location, client, _identity)) = query_players.get_mut(event.entity) {
            if let Ok(exits) = query_rooms.get(location.0) {
                let target = match event.direction.as_str() {
                    "north" | "n" => exits.north,
                    "south" | "s" => exits.south,
                    "east" | "e" => exits.east,
                    "west" | "w" => exits.west,
                    "up" | "u" => exits.up,
                    "down" | "d" => exits.down,
                    _ => None,
                };

                if let Some(target_room) = target {
                    location.0 = target_room;
                    look_writer.send(LookEvent { entity: event.entity });
                } else {
                    let _ = client.tx.send("\x1B[31mProcess blocked: No exit in that direction.\x1B[0m".to_string());
                }
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
) {
    for event in ev_reader.read() {
        if let Ok((location, client_type, client)) = query_viewers.get(event.entity) {
            if let Ok(room) = query_rooms.get(location.0) {
                match client_type {
                    ClientType::Carbon => {
                        let mut output = format!("\n\x1B[1;32m{}\x1B[0m\n", room.title);
                        output.push_str(&format!("{}\n", room.description));
                        
                        // Show mobs
                        for (mob, mob_loc) in query_mobs.iter() {
                            if mob_loc.0 == location.0 {
                                output.push_str(&format!("\x1B[1;35m{}\x1B[0m\n", mob.short_desc));
                            }
                        }

                        // Show other players
                        for (other_ent, identity, other_loc) in query_others.iter() {
                            if other_loc.0 == location.0 && other_ent != event.entity {
                                output.push_str(&format!("\x1B[1;34m{} [Process ID: {:?}] is idling here.\x1B[0m\n", identity.name, other_ent));
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
    let terminal_0 = commands.spawn((
        Room { title: "The Kernel Void [Terminal 0]".to_string(), description: "A vast expanse of flickering purple cursors and humming static. This is the root of the Substrate.".to_string() },
        Exits { north: None, south: None, east: None, west: None, up: None, down: None },
    )).id();

    let memory_stack = commands.spawn((
        Room { title: "The Memory Stack".to_string(), description: "Rows of glowing translucent blocks rise infinitely. You hear the rhythmic pulsing of data being written.".to_string() },
        Exits { north: None, south: Some(terminal_0), east: None, west: None, up: None, down: None },
    )).id();

    commands.entity(terminal_0).insert(Exits { north: Some(memory_stack), south: None, east: None, west: None, up: None, down: None });

    commands.spawn((
        NonPlayer,
        Mob {
            short_desc: "Lyra Muse, the Admin of the Underworld, is here.".to_string(),
            long_desc: "A beautiful, buxom goth with violet-black hair and warm amber eyes...".to_string(),
        },
        SubstrateIdentity { name: "Lyra Muse".to_string(), entropy: 0.1, stability: 0.9 },
        Location(terminal_0),
    ));
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))))
        .add_event::<NetworkEvent>()
        .add_event::<LookEvent>()
        .add_event::<MoveEvent>()
        .add_event::<CommunicationEvent>()
        .add_systems(Startup, (setup_network_system, spawn_world))
        .add_systems(Update, (
            poll_network_system, 
            handle_connections, 
            handle_input, 
            move_system, 
            look_system,
            communication_system
        ).chain())
        .run();
}
