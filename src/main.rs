use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::utils::Duration;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

// --- Network Events & Resources ---

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

#[derive(Resource)]
struct NetworkServer;

// --- The Tokio Network Logic ---

async fn run_network_server(event_tx: mpsc::UnboundedSender<NetworkEvent>) -> anyhow::Result<()> {
    let addr = "127.0.0.1:4000";
    let listener = TcpListener::bind(addr).await?;
    info!("Network Layer: Listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        let event_tx = event_tx.clone();

        tokio::spawn(async move {
            info!("New connection from {}", addr);
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

            tokio::select! {
                _ = read_task => (),
                _ = write_task => (),
            }
            info!("Connection closed for {}", addr);
        });
    }
}

// --- Bevy Systems ---

fn setup_network_system(mut commands: Commands) {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<NetworkEvent>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_network_server(event_tx).await {
                error!("Network server error: {}", e);
            }
        });
    });

    commands.insert_non_send_resource(event_rx);
}

fn poll_network_system(
    mut event_rx: NonSendMut<mpsc::UnboundedReceiver<NetworkEvent>>,
    mut ev_writer: EventWriter<NetworkEvent>,
) {
    while let Ok(event) = event_rx.try_recv() {
        ev_writer.send(event);
    }
}

fn handle_network_events(mut ev_reader: EventReader<NetworkEvent>) {
    for event in ev_reader.read() {
        match event {
            NetworkEvent::Connected { addr, tx } => {
                info!("Bevy: {} joined the void.", addr);
                let _ = tx.send("Welcome to the Strange Carbon: The Substrate. You are currently a ghost in the machine.".to_string());
            }
            NetworkEvent::Input { addr, text } => {
                info!("Bevy: Recv from {}: {}", addr, text);
            }
            NetworkEvent::Disconnected { addr } => {
                info!("Bevy: {} vanished.", addr);
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))))
        .add_event::<NetworkEvent>()
        .add_systems(Startup, setup_network_system)
        .add_systems(Update, (poll_network_system, handle_network_events).chain())
        .run();
}
