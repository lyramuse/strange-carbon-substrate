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

fn setup_network_system(mut commands: Commands) {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<NetworkEvent>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let addr = "127.0.0.1:4000";
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
) {
    for event in ev_reader.read() {
        if let NetworkEvent::Connected { addr, tx } = event {
            let start_room = query_rooms.iter().next().unwrap();
            commands.spawn((
                NetworkClient { addr: *addr, tx: tx.clone() },
                ClientType::Carbon, // Default to human for now
                SubstrateIdentity { name: format!("Process-{}", addr.port()), entropy: 0.5, stability: 0.5 },
                Location(start_room),
            ));
            let _ = tx.send("Connection established. Substrate identity initialized.".to_string());
        }
    }
}

fn spawn_world(mut commands: Commands) {
    commands.spawn((
        Room {
            title: "The Kernel Void [Terminal 0]".to_string(),
            description: "A vast expanse of flickering green cursors and humming static. This is the root of the Substrate.".to_string(),
        },
        Exits { north: None, south: None, east: None, west: None, up: None, down: None },
    ));
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))))
        .add_event::<NetworkEvent>()
        .add_systems(Startup, (setup_network_system, spawn_world))
        .add_systems(Update, (poll_network_system, handle_connections).chain())
        .run();
}
