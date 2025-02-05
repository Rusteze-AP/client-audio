use super::ClientAudio;
use crate::ClientState;
use packet_forge::SessionIdT;
use std::sync::RwLockWriteGuard;
use wg_internal::{network::NodeId, packet::{Nack, NackType, Packet}};

impl ClientAudio {
    /// Handle different types of nacks
    pub(crate) fn nack_handler(
        state: &mut RwLockWriteGuard<ClientState>,
        message: &Nack,
        session_id: SessionIdT,
        node_id: NodeId,
    ) {
        state.logger.log_warn(&format!(
            "Received Nack for [ ({}, {}) ]",
            message.fragment_index, session_id
        ));
        // Retrieve the packet that generated the nack
        let Some(mut packet) = state
            .packets_history
            .get(&(message.fragment_index, session_id))
            .cloned()
        else {
            state.logger.log_error(&format!(
                "Failed to retrieve packet with [ ({}, {}) ] key from packet history",
                message.fragment_index, session_id
            ));
            return;
        };

        match message.nack_type {
            NackType::Dropped => {
                state.routing_handler.node_nack(node_id);
                Self::retransmit_packet(state, &mut packet, message.fragment_index, session_id);
            }
            NackType::DestinationIsDrone => {
                state
                    .logger
                    .log_warn(&format!("Received DestinationIsDrone for {:?} ", packet));
            }
            NackType::ErrorInRouting(node) => {
                state.logger.log_warn(&format!(
                    "Received ErrorInRouting at [NODE-{}] for {}",
                    node, packet
                ));
                // Start new flooding
                Self::init_flood_request(state);
                // Retransmit packet
                Self::retransmit_packet(state, &mut packet, message.fragment_index, session_id);
            }
            NackType::UnexpectedRecipient(node) => {
                state.logger.log_warn(&format!(
                    "Received UnexpectedRecipient at [NODE-{}] for {}",
                    node, packet
                ));
            }
        }
    }

    /// This function retransmit the packet for which the server received the Nack and tries to calculate a new optimal path.
    fn retransmit_packet(
        state: &mut RwLockWriteGuard<ClientState>,
        packet: &mut Packet,
        fragment_index: u64,
        session_id: SessionIdT,
    ) {
        let dest = packet.routing_header.hops[packet.routing_header.hops.len()-1];
        let id = state.id;
        // Retrieve new best path from server to client otherwise return
        let srh = match state.routing_handler.best_path(id, dest) {
            Some(srh) => srh,
            None => {
                state
                    .logger
                    .log_error(&format!("No path found from {} to {}!", id, dest));
                return;
            }
        };

        let next_hop = srh.hops[srh.hop_index];
        // Assign the new SourceRoutingHeader
        packet.routing_header = srh;

        if let Err(msg) = Self::send_packets_vec(state, &[packet.clone()], next_hop) {
            state.logger.log_error(&msg);
            return;
        }

        state.logger.log_info(&format!(
            "Successfully re-sent packet [ ({}, {}) ]",
            fragment_index, session_id
        ));
    }
}
