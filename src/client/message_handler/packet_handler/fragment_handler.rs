use super::ClientAudio;
use crate::{ClientState, Status};
use packet_forge::{FileMetadata, Index, MessageType};
use std::sync::RwLockWriteGuard;
use wg_internal::packet::{Fragment, Packet};

impl ClientAudio {
    /// Handles the fragment of a packet
    pub(crate) fn fragment_handler(
        state: &mut RwLockWriteGuard<ClientState>,
        fragment: &Fragment,
        packet: &Packet,
    ) {
        let client_id = packet.routing_header.hops[0];
        let key = (client_id, packet.session_id);

        // Save fragment
        let total_fragments = fragment.total_n_fragments;
        state
            .packets_map
            .entry(key)
            .or_default()
            .push(fragment.clone());

        let mut fragments = state.packets_map.get(&key).unwrap().clone();
        // Send Ack back to the Client
        Self::send_ack(state, packet, fragment.fragment_index);

        // If all fragments are received, assemble the message
        if fragments.len() as u64 == total_fragments {
            let assembled = match state.packet_forge.assemble_dynamic(&mut fragments) {
                Ok(message) => message,
                Err(e) => {
                    state.logger.log_error(&format!(
                        "An error occurred when assembling fragments: {}",
                        e
                    ));
                    return;
                }
            };
            // menage the entire message
            Self::handle_node_message(state, assembled);
        }
    }

    /// Handles the message received from the node
    pub(crate) fn handle_node_message(
        state: &mut RwLockWriteGuard<ClientState>,
        message: MessageType,
    ) {
        match message {
            // When the chunk response is received, put the chunk in the buffer and send the event to the rocket endpoint
            MessageType::ChunkResponse(chunk) => {
                state.logger.log_info(&format!(
                    "Received chunk response for file {}",
                    chunk.file_hash
                ));
                // get the channel corresponding to the chunk
                let sender = state
                    .inner_senders
                    .get(&(chunk.file_hash, chunk.chunk_index))
                    .cloned();

                match sender {
                    Some(sender) => {
                        // Save chunk in the buffer for rocket
                        state.song_map.insert(
                            (chunk.file_hash, chunk.chunk_index),
                            chunk.chunk_data.to_vec(),
                        );
                        // send the event to the rocket server
                        sender.send(true).unwrap();
                    }
                    None => {
                        state.logger.log_error(&format!(
                            "No inner sender found for file {}",
                            chunk.file_hash
                        ));
                    }
                }
                chunk.file_hash;
            }
            // When the file list is received, insert the new songs metadata in the database
            MessageType::ResponseFileList(list) => {
                for song in list.file_list {
                    match song {
                        FileMetadata::Song(song) => {
                            state
                                .logger
                                .log_info(&format!("Received song metadata: {}", song.id));
                            if let Err(e) = state.db.insert_song_meta(song) {
                                state
                                    .logger
                                    .log_error(&format!("Failed to insert song metadata: {}", e));
                            }
                        }
                        // this client does not handle video files
                        FileMetadata::Video(video) => {
                            state.logger.log_info(&format!(
                                "Received video metadata: {} - Drop it",
                                video.id
                            ));
                        }
                    }
                }
                // if the client is not already running, set the status to running
                state.status = Status::Running;
            }
            // When the peer list is received, insert the peer in the client_song_map and send the first segment request to the node
            MessageType::ResponsePeerList(list) => {
                state
                    .logger
                    .log_info(&format!("Received peer list {:?}", list));
                state
                    .client_song_map
                    .insert(list.file_hash, list.peers[0].client_id);
                Self::send_internal_segment_request(state, list.file_hash, 0);
            }
            // When a peer asks for a chunk, send the chunk response to the node
            MessageType::ChunkRequest(chunk) => {
                state.logger.log_info(&format!(
                    "Received chunk request for file {}",
                    chunk.file_hash
                ));
                if let Index::Indexes(vec) = &chunk.chunk_index {
                    for chunk_index in vec {
                        Self::send_chunk_response(state, chunk.file_hash, *chunk_index, chunk.client_id);
                    }
                }
                else{
                    state.logger.log_error("Invalid chunk index");
                }
            }
            _ => {
                state
                    .logger
                    .log_error(&format!("Unknown message type: {:?}", message));
            }
        }
    }
}
