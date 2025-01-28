use super::Client;
use crate::{ClientState, Status};
use packet_forge::{FileMetadata, MessageType};
use std::sync::RwLockWriteGuard;
use wg_internal::packet::{Fragment, Packet};

impl Client {
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

        let fragments = state.packets_map.get(&key).unwrap().clone();
        // Send Ack back to the Client
        Self::send_ack(state, packet, fragment.fragment_index);

        // If all fragments are received, assemble the message
        if fragments.len() as u64 == total_fragments {
            let assembled = match state.packet_forge.assemble_dynamic(fragments.clone()) {
                Ok(message) => message,
                Err(e) => {
                    state.logger.log_error(&format!(
                        "An error occurred when assembling fragments: {}",
                        e
                    ));
                    return;
                }
            };

            Self::handle_node_message(state, assembled);
        }
    }

    pub(crate) fn handle_node_message(
        state: &mut RwLockWriteGuard<ClientState>,
        message: MessageType,
    ) {
        match message {
            MessageType::ChunkResponse(chunk) => {
                state.logger.log_info(&format!(
                    "Received chunk response for file {}",
                    chunk.file_hash
                ));

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
                        FileMetadata::Video(video) => {
                            state.logger.log_info(&format!(
                                "Received video metadata: {} - Drop it",
                                video.id
                            ));
                        }
                    }
                }
                state.status = Status::Running;
            }
            _ => {
                state
                    .logger
                    .log_error(&format!("Unknown message type: {:?}", message));
            }
        }
    }
}
