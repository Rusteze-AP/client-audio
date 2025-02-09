use super::ClientAudio;
use crate::ClientState;
use bytes::Bytes;
use packet_forge::{FileMetadata, MessageType, RequestFileList, SubscribeClient};
use std::sync::RwLockWriteGuard;
use wg_internal::network::NodeId;

impl ClientAudio {
    /// Send subscribe message to the server
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

        let _ = Self::send_message(state, message, id, server_id);
    }

    /// Send request file list message to the server
    pub(crate) fn send_request_filelist(state: &mut RwLockWriteGuard<ClientState>) {
        let id = state.id;
        let server_id = state.servers_id[0];

        let message = MessageType::RequestFileList(RequestFileList::new(id));
        let _ = Self::send_message(state, message, id, server_id);
    }

    // Send a chunk response to the destination node. Used in peer-to-peer communication.
    pub(crate) fn send_chunk_response(
        state: &mut RwLockWriteGuard<ClientState>,
        file_id: u16,
        segment: u32,
        dst: NodeId,
    ) {
        let id = state.id;
        let payload = match state.db.get_song_segment(file_id, segment) {
            Ok(chunk) => chunk,
            Err(e) => {
                state.logger.log_error(&e);
                return;
            }
        };
        let chunk_data = Bytes::from(payload);

        let message = MessageType::ChunkResponse(packet_forge::ChunkResponse::new(
            file_id, segment, 0, chunk_data,
        ));

        let _ = Self::send_message(state, message, id, dst);
    }

    // Send a segment request to the destination node. Used by the thread after receving the peer list.
    pub(crate) fn send_internal_segment_request(
        state: &mut RwLockWriteGuard<ClientState>,
        file_id: u16,
        segment: u32,
    ) {
        let id = state.id;

        let dst = *state.client_song_map.get(&file_id).unwrap();
        let message = MessageType::ChunkRequest(packet_forge::ChunkRequest::new(
            id,
            file_id,
            packet_forge::Index::Indexes(vec![segment]),
        ));

        match Self::send_message(state, message, id, dst) {
            Ok(()) => {
                state
                    .logger
                    .log_info(&format!("Successfully sent segment request"));
            }
            Err(()) => {
                let sender = state.inner_senders.get(&(file_id, segment)).cloned();

                match sender {
                    Some(sender) => {
                        // send the event to the rocket server
                        sender.send(false).unwrap();
                    }
                    None => {
                        state.logger.log_error(&format!(
                            "No inner sender found for file {} to send error",
                            file_id
                        ));
                    }
                }

                state
                    .logger
                    .log_error(&format!("Failed to send segment request"));
            }
        }
    }

    /// Send a segment request to the destination node. Used by the rocket server.
    pub(crate) fn send_segment_request(&mut self, file_id: u16, segment: u32) {
        let mut state = self.state.write().unwrap();
        let id = state.id;
        let server_id = state.servers_id[0];

        // if the playlist is needed send peer list request to server
        if segment == 0 {
            let message = MessageType::RequestPeerList(packet_forge::RequestPeerList {
                client_id: id,
                file_hash: file_id,
            });

            match Self::send_message(&mut state, message, id, server_id) {
                Ok(()) => {
                    state
                        .logger
                        .log_info(&format!("Successfully sent segment request"));
                }
                Err(()) => {
                    let sender = state.inner_senders.get(&(file_id, segment)).cloned();

                    match sender {
                        Some(sender) => {
                            // send the event to the rocket server
                            sender.send(false).unwrap();
                        }
                        None => {
                            state.logger.log_error(&format!(
                                "No inner sender found for file {} to send error",
                                file_id
                            ));
                        }
                    }

                    state
                        .logger
                        .log_error(&format!("Failed to send segment request"));
                }
            }
            return;
        } else {
            // else send the segment request to the peer

            match state.client_song_map.get(&file_id) {
                None => {
                    state
                        .logger
                        .log_error(&format!("No client found for file {}", file_id));
                    return;
                }
                Some(dst) => {
                    let message = MessageType::ChunkRequest(packet_forge::ChunkRequest::new(
                        id,
                        file_id,
                        packet_forge::Index::Indexes(vec![segment]),
                    ));

                    let dst_clone = *dst;
                    match Self::send_message(&mut state, message, id, dst_clone) {
                        Ok(()) => {
                            state
                                .logger
                                .log_info(&format!("Successfully sent segment request"));
                        }
                        Err(()) => {
                            let sender = state.inner_senders.get(&(file_id, segment)).cloned();

                            match sender {
                                Some(sender) => {
                                    // send the event to the rocket server
                                    sender.send(false).unwrap();
                                }
                                None => {
                                    state.logger.log_error(&format!(
                                        "No inner sender found for file {} to send error",
                                        file_id
                                    ));
                                }
                            }

                            state
                                .logger
                                .log_error(&format!("Failed to send segment request"));
                        }
                    }
                }
            }
        }
    }

    /// Send a packet with the path to the destination node
    pub(crate) fn send_message(
        state: &mut RwLockWriteGuard<ClientState>,
        message: MessageType,
        src: NodeId,
        dst: NodeId,
    ) -> Result<(), ()> {
        // for logging purposes
        let message_type = match message {
            MessageType::SubscribeClient(_) => "SubscribeClient",
            MessageType::UnsubscribeClient(_) => "UnsubscribeClient",
            MessageType::RequestFileList(_) => "RequestFileList",
            MessageType::ResponseFileList(_) => "ResponseFileList",
            MessageType::ChunkRequest(_) => "ChunkRequest",
            MessageType::ChunkResponse(_) => "ChunkResponse",
            _ => "Unknown",
        };

        //Compute the best path
        let srh = match state.routing_handler.best_path(src, dst) {
            Some(srh) => srh,
            None => {
                state
                    .logger
                    .log_error(&format!("No path found from {} to {}!", src, dst));
                return Err(());
            }
        };

        // disassemble the message into frames of the correct size
        let frames = match state.packet_forge.disassemble(message, &srh) {
            Ok(frames) => frames,
            Err(e) => {
                state
                    .logger
                    .log_error(&format!("error on node messge disassemble: {}", e));
                return Err(());
            }
        };

        // send all the frames to the next hop
        if let Err(e) = Self::send_packets_vec(state, &frames, srh.current_hop().unwrap()) {
            state.logger.log_error(&e);
            return Err(());
        }

        state
            .logger
            .log_info(&format!("Successfully sent message {}", message_type));
        Ok(())
    }
}
