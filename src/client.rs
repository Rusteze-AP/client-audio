mod message_handler;
use crate::client_endpoints::{audio_files, get_id, get_song, is_ready};
use crate::database::AudioDatabase;
use crossbeam::channel::{Receiver, Sender};
use logger::{LogLevel, Logger};
use packet_forge::ClientT;
use packet_forge::{FileHash, PacketForge, SessionIdT};
use rocket::fs::relative;
use rocket::{Error, Ignite, Rocket};
use routing_handler::RoutingHandler;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wg_internal::controller::{DroneCommand, DroneEvent};
use wg_internal::network::NodeId;
use wg_internal::packet::{Fragment, Packet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Starting,
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
pub struct ClientAudio {
    pub state: Arc<RwLock<ClientState>>,
}

impl ClientT for ClientAudio {
    fn new(
        id: NodeId,
        command_send: Sender<DroneEvent>,
        command_recv: Receiver<DroneCommand>,
        receiver: Receiver<Packet>,
        senders: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        let db_path = &format!("db/client_audio/client-{}", id);
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
            status: Status::Starting,
            db: AudioDatabase::new(db_path),
            logger: Logger::new(LogLevel::All as u8, false, format!("audio_client_{}", id)),
            routing_handler: RoutingHandler::new(),
            packets_map: HashMap::new(),
            packets_history: HashMap::new(),
            song_map: HashMap::new(),
        };

        ClientAudio {
            state: Arc::new(RwLock::new(state)),
        }
    }

    fn run(self: Box<Self>, init_client_path: &str) {
        // Initialize the database
        match self.state.read().unwrap().db.init(init_client_path) {
            Ok(_) => self.state.read().unwrap().logger.log_info("Database initialized"),
            Err(e) => self.state.read().unwrap().logger.log_error(e.as_str()),
        }
    
        let _processing_handle = self.clone().start_message_processing();
    
        // Create a new Tokio runtime and block on `configure`
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async move {
            if let Err(e) = Self::configure(*self).await {
                eprintln!("Failed to configure client: {}", e);
            }
        });
    }

    fn get_id(&self) -> NodeId {
        match self.state.read() {
            Ok(state) => state.id,
            Err(e) => {
                eprintln!("Error reading state {}", e);
                0
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ClientAudio {
    #[must_use]
    async fn configure(client: ClientAudio) -> Result<Rocket<Ignite>, Error> {
        let id = client.get_id();
        rocket::build()
            .manage(client)
            .configure(rocket::Config::figment().merge(("port", 8000 + id as u16)))
            .mount("/", routes![audio_files, get_song, is_ready, get_id])
            .mount("/", rocket::fs::FileServer::from(relative!("static")))
            .launch()
            .await
    }
}
