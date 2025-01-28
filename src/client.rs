mod message_handler;
use crate::client_endpoints::{audio_files, get_id, get_song, is_ready};
use crate::database::AudioDatabase;
use crossbeam::channel::{Receiver, Sender};
use logger::{LogLevel, Logger};
use packet_forge::{FileHash, PacketForge, SessionIdT};
use rocket::fs::relative;
use rocket::time::error::DifferentVariant;
use std::time::Duration;
use rocket::{self, Build, Ignite, Rocket};
use routing_handler::RoutingHandler;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use wg_internal::controller::{DroneCommand, DroneEvent};
use wg_internal::network::NodeId;
use wg_internal::packet::{Fragment, Packet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
    Terminated,
}

pub struct ClientState {
    pub id: NodeId,
    pub flood_id: u64,
    pub servers_id: Vec<NodeId>,
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub senders: HashMap<NodeId, Sender<Packet>>,
    pub inner_senders: HashMap<(FileHash, u32), Sender<bool>>,
    pub packet_forge: PacketForge,
    pub status: Status,
    pub db: AudioDatabase,
    pub logger: Logger,
    pub routing_handler: RoutingHandler,
    pub packets_map: HashMap<(NodeId, SessionIdT), Vec<Fragment>>,
    pub song_map: HashMap<(FileHash, u32), Vec<u8>>,
    pub packets_history: HashMap<(u64, SessionIdT), Packet>,
}

#[derive(Clone)]
pub struct Client {
    pub state: Arc<RwLock<ClientState>>,
}

impl Client {
    #[must_use]
    pub fn new(
        id: NodeId,
        command_send: Sender<DroneEvent>,
        command_recv: Receiver<DroneCommand>,
        receiver: Receiver<Packet>,
        senders: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        let db_path = &format!("initialization_files/client_audio/client-{}/db", id);
        let state = ClientState {
            id,
            flood_id: 0,
            servers_id: Vec::new(),
            controller_send: command_send,
            controller_recv: command_recv,
            packet_recv: receiver,
            senders,
            inner_senders: HashMap::new(),
            packet_forge: PacketForge::new(),
            status: Status::Idle,
            db: AudioDatabase::new(db_path),
            logger: Logger::new(LogLevel::All as u8, false, format!("audio_client_{}", id)),
            routing_handler: RoutingHandler::new(),
            packets_map: HashMap::new(),
            packets_history: HashMap::new(),
            song_map: HashMap::new(),
        };

        let local_path = &format!("initialization_files/client_audio/client-{}", id);
        // Initialize the database
        match state.db.init(local_path) {
            Ok(_) => state.logger.log_info("database initialized"),
            Err(e) => state.logger.log_error(e.as_str()),
        }

        Client {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn get_id(&self) -> NodeId {
        match self.state.read() {
            Ok(state) => state.id,
            Err(e) => {
                eprintln!("Error reading state {}", e);
                0
            }
        }
    }

    #[must_use]
    fn configure(client: Client) -> Rocket<Build> {
        let id = client.get_id();
        rocket::build()
            .manage(client)
            .configure(rocket::Config::figment().merge(("port", 8000 + id as u16)))
            .mount("/", routes![audio_files, get_song, is_ready, get_id])
            .mount("/", rocket::fs::FileServer::from(relative!("static")))
    }

    pub async fn run(self) -> Result<Rocket<Ignite>, rocket::Error> {
        let _processing_handle = self.clone().start_message_processing();
        Self::configure(self).launch().await
    }
}
