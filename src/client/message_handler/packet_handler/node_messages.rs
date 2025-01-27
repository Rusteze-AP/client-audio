use super::Client;
use crate::ClientState;
use packet_forge::{
    FileMetadata, MessageType, RequestFileList, SubscribeClient, UnsubscribeClient,
};
use std::sync::RwLockWriteGuard;
use wg_internal::network::NodeId;

impl Client {
    pub(crate) fn send_subscribe(state: &mut RwLockWriteGuard<ClientState>) {
        let id = state.id;
        let server_id = state.server_id;

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
        let server_id = state.server_id;

        let message = MessageType::UnsubscribeClient(UnsubscribeClient::new(id));

        Self::send_message(state, message, id, server_id);
    }

    pub(crate) fn send_update_filelist(state: &mut RwLockWriteGuard<ClientState>) {
        //TODO mabe to implement
    }

    pub(crate) fn send_request_filelist(state: &mut RwLockWriteGuard<ClientState>) {
        let id = state.id;
        let server_id = state.server_id;

        let message = MessageType::RequestFileList(RequestFileList::new(id));
        Self::send_message(state, message, id, server_id);
    }

    pub(crate) fn send_message(
        state: &mut RwLockWriteGuard<ClientState>,
        message: MessageType,
        src: NodeId,
        dst: NodeId,
    ) {
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

        if let Err(e) = Self::send_packets_vec(state, &frames, srh.next_hop().unwrap()) {
            state.logger.log_error(&e);
            return;
        }

        state
            .logger
            .log_info(&format!("Successfully sent node message"));
    }
}
