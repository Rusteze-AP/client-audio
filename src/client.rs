use crossbeam::channel::{Receiver, Sender};
use logger::{LogLevel, Logger};
use packet_forge::PacketForge;
use rocket::fs::relative;
use rocket::{self, Build, Ignite, Rocket};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use wg_internal::controller::{DroneCommand, DroneEvent};
use wg_internal::network::NodeId;
use wg_internal::packet::Packet;

use crate::client_endpoints::{audio_files, stream_audio};
use crate::database::AudioDatabase;

pub struct ClientState {
    pub id: NodeId,
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub senders: HashMap<NodeId, Sender<Packet>>,
    pub packet_forge: PacketForge,
    pub terminated: bool,
    pub db: AudioDatabase,
    pub logger: Logger,
}

#[derive(Clone)]
pub struct Client {
    state: Arc<RwLock<ClientState>>,
}

impl Client {
    #[must_use]
    pub fn new(
        id: NodeId,
        command_send: Sender<DroneEvent>,
        command_recv: Receiver<DroneCommand>,
        receiver: Receiver<Packet>,
        senders: HashMap<NodeId, Sender<Packet>>,
        database_name: &str,
    ) -> Self {
        let state = ClientState {
            id,
            controller_send: command_send,
            controller_recv: command_recv,
            packet_recv: receiver,
            senders,
            packet_forge: PacketForge::new(),
            terminated: false,
            db: AudioDatabase::new(database_name),
            logger: Logger::new(LogLevel::All as u8, false, "audio_client".to_string()),
        };

        Client {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn get_id(&self) -> NodeId {
        println!("get iddddd");
        match self.state.read() {
            Ok(state) => state.id,
            Err(e) => {
                eprintln!("Error reading state {}", e);
                0
            }
        }
    }

    fn command_dispatcher(&self, command: &DroneCommand) {
        let mut state = self.state.write().unwrap();

        match command {
            DroneCommand::Crash => {
                state.terminated = true;
            }
            DroneCommand::SetPacketDropRate(_) => {
                eprintln!(
                    "Client {}, error: received a SetPacketDropRate command",
                    state.id
                );
            }
            _ => {
                eprintln!(
                    "Client {}, error: received an unknown command: {:?}",
                    state.id, command
                );
            }
        }
    }

    #[must_use]
    pub fn start_message_processing(self) -> thread::JoinHandle<()> {
        let state = self.state.clone();

        println!("from message client id {}", state.read().unwrap().id);
        match state.read().unwrap().db.init("assets") {
            Ok(_) => state
                .read()
                .unwrap()
                .logger
                .log_info("database initialized"),
            Err(e) => state.read().unwrap().logger.log_error(e.as_str()),
        }

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
        })
    }

    #[must_use]
    fn configure(client: Client) -> Rocket<Build> {
        rocket::build()
            .manage(client.state.clone())
            .mount("/", routes![audio_files, stream_audio])
            .mount("/", rocket::fs::FileServer::from(relative!("static")))
    }

    pub async fn run(self) -> Result<Rocket<Ignite>, rocket::Error> {
        let _processing_handle = self.clone().start_message_processing();
        Self::configure(self).launch().await
    }
}
