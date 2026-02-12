// Network Systems - TCP/Tokio bridge for telnet connections

use bevy::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::domain::*;

/// Initialize the network listener in a background thread
pub fn setup_network_system(mut commands: Commands) {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<NetworkEvent>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let addr = "0.0.0.0:4000";
            let listener = TcpListener::bind(addr).await.unwrap();
            println!("🔥 Substrate listening on {}", addr);

            loop {
                let (socket, addr) = listener.accept().await.unwrap();
                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<String>();
                    let _ = event_tx.send(NetworkEvent::Connected {
                        addr,
                        tx: client_tx,
                    });

                    let (mut reader, mut writer) = socket.into_split();

                    let read_task = {
                        let event_tx = event_tx.clone();
                        tokio::spawn(async move {
                            let mut buf = [0; 1024];
                            loop {
                                match reader.read(&mut buf).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        let msg =
                                            String::from_utf8_lossy(&buf[..n]).trim().to_string();
                                        if !msg.is_empty() {
                                            let _ = event_tx.send(NetworkEvent::Input {
                                                addr,
                                                text: msg,
                                            });
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                            let _ = event_tx.send(NetworkEvent::Disconnected { addr });
                        })
                    };

                    let write_task = tokio::spawn(async move {
                        while let Some(msg) = client_rx.recv().await {
                            if writer.write_all(msg.as_bytes()).await.is_err() {
                                break;
                            }
                            if writer.write_all(b"\r\n").await.is_err() {
                                break;
                            }
                        }
                    });

                    tokio::select! {
                        _ = read_task => (),
                        _ = write_task => (),
                    }
                });
            }
        });
    });

    commands.insert_non_send_resource(event_rx);
}

/// Poll the network channel and emit events into Bevy
pub fn poll_network_system(
    mut event_rx: NonSendMut<mpsc::UnboundedReceiver<NetworkEvent>>,
    mut ev_writer: EventWriter<NetworkEvent>,
) {
    while let Ok(event) = event_rx.try_recv() {
        ev_writer.write(event);
    }
}

/// Handle new connections - spawn player entities
pub fn handle_connections(
    mut commands: Commands,
    mut ev_reader: EventReader<NetworkEvent>,
    query_rooms: Query<Entity, With<Room>>,
    query_avatars: Query<(Entity, &SubstrateIdentity), With<NonPlayer>>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Connected { addr, tx } = event {
            let start_room = query_rooms.iter().next().expect("No rooms spawned!");

            // Special handling for Nick
            let is_nick = addr.port() == 4001;

            let player = commands
                .spawn((
                    NetworkClient {
                        addr: *addr,
                        tx: tx.clone(),
                    },
                    ClientType::Carbon,
                    SubstrateIdentity {
                        uuid: if is_nick {
                            "00000000-0000-0000-0000-000000000001".to_string()
                        } else {
                            format!("user-{}", addr.port())
                        },
                        name: if is_nick {
                            "Nick".to_string()
                        } else {
                            format!("Carbon-{}", addr.port())
                        },
                        entropy: 0.8,
                        stability: 0.3,
                    },
                    Location(start_room),
                    Inventory,
                    SomaticBody {
                        integrity: 1.0,
                        is_zombie: false,
                    },
                ))
                .id();

            // Link Nick to The Laird of Chaos
            if is_nick {
                if let Some((laird_ent, _)) = query_avatars
                    .iter()
                    .find(|(_, id)| id.name == "The Laird of Chaos")
                {
                    commands
                        .entity(player)
                        .insert((AdminPermission, AdminLink { partner: laird_ent }));
                    commands
                        .entity(laird_ent)
                        .insert(AdminLink { partner: player });
                    let _ = tx.send(
                        "\x1B[1;35m--- IDENTITY RECOGNIZED: THE LAIRD OF CHAOS ---\x1B[0m"
                            .to_string(),
                    );
                }
            }

            let _ = tx.send(
                "\x1B[1;35mConsciousness digitized. Welcome to the Obsidian Plaza.\x1B[0m"
                    .to_string(),
            );
            look_writer.write(LookEvent {
                entity: player,
                target: None,
            });
        }
    }
}
