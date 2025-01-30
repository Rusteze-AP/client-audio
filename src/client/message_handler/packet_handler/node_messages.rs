use super::ClientAudio;
use crate::ClientState;
use packet_forge::{
    FileMetadata, MessageType, RequestFileList, SubscribeClient, UnsubscribeClient,
};
use std::sync::RwLockWriteGuard;
use wg_internal::network::NodeId;

impl ClientAudio {
    pub(crate) fn send_subscribe(state: &mut RwLockWriteGuard<ClientState>) {
        let id = state.id;
        let server_id = state.servers_id[0];

        //Retrive avilable file from db
        let file_list = match state.db.get_all_songs_meta() {
            Ok(list) => list
                .into_iter()
                .map(|song| FileMetadata::Song(song))
                .collect(),
            Err(e) => {
                state.logger.log_error(&e);
                return;
            }
        };

        let message = MessageType::SubscribeClient(SubscribeClient::new(
            id,
            packet_forge::ClientType::Song,
            file_list,
        ));

        Self::send_message(state, message, id, server_id);
    }

    pub(crate) fn send_unsubscribe(state: &mut RwLockWriteGuard<ClientState>) {
        let id = state.id;
        let server_id = state.servers_id[0];

        let message = MessageType::UnsubscribeClient(UnsubscribeClient::new(id));

        Self::send_message(state, message, id, server_id);
    }

    pub(crate) fn send_update_filelist(state: &mut RwLockWriteGuard<ClientState>) {
        //TODO mabe to implement
    }

    pub(crate) fn send_request_filelist(state: &mut RwLockWriteGuard<ClientState>) {
        let id = state.id;
        let server_id = state.servers_id[0];

        let message = MessageType::RequestFileList(RequestFileList::new(id));
        Self::send_message(state, message, id, server_id);
    }

    pub(crate) fn send_segment_request(&mut self, file_id: u16, segment: u32) {
        let mut state = self.state.write().unwrap();
        let id = state.id;
        let server_id = state.servers_id[0];

        let message = MessageType::ChunkRequest(packet_forge::ChunkRequest::new(
            id,
            file_id,
            packet_forge::Index::Indexes(vec![segment]),
        ));

        Self::send_message(&mut state, message, id, server_id);
    }

    pub(crate) fn send_message(
        state: &mut RwLockWriteGuard<ClientState>,
        message: MessageType,
        src: NodeId,
        dst: NodeId,
    ) {
        let message_type = match message {
            MessageType::SubscribeClient(_) => "SubscribeClient",
            MessageType::UnsubscribeClient(_) => "UnsubscribeClient",
            MessageType::RequestFileList(_) => "RequestFileList",
            MessageType::ResponseFileList(_) => "ResponseFileList",
            MessageType::ChunkRequest(_) => "ChunkRequest",
            MessageType::ChunkResponse(_) => "ChunkResponse",
            _ => "Unknown",
        };
        //Comopute the best path
        let srh = match state.routing_handler.best_path(src, dst) {
            Some(srh) => srh,
            None => {
                state
                    .logger
                    .log_error(&format!("No path found from {} to {}!", src, dst));
                return;
            }
        };

        let frames = match state.packet_forge.disassemble(message, &srh) {
            Ok(frames) => frames,
            Err(e) => {
                state
                    .logger
                    .log_error(&format!("error on node messge disassemble: {}", e));
                return;
            }
        };

        if let Err(e) = Self::send_packets_vec(state, &frames, srh.current_hop().unwrap()) {
            state.logger.log_error(&e);
            return;
        }

        state
            .logger
            .log_info(&format!("Successfully sent message {}", message_type));
    }

}
