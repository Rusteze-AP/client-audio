use super::ClientAudio;
use crate::ClientState;
use std::sync::RwLockWriteGuard;
use wg_internal::{controller::DroneEvent, packet::Packet};

impl ClientAudio {
    pub(crate) fn ack_handler(
        state: &mut RwLockWriteGuard<ClientState>,
        session_id: u64,
        fragment_index: u64,
    ) {
        let Some(_) = state.packets_history.remove(&(fragment_index, session_id)) else {
            state.logger.log_error(&format!(
                "Failed to remove [ ({}, {}) ] key from packet history",
                fragment_index, session_id
            ));
            return;
        };
    }

    // Builds and sends an `Ack` to the `next_hop`. If it fails it tries to use the Simulation Controller
    pub(crate) fn send_ack(
        state: &mut RwLockWriteGuard<ClientState>,
        packet: &Packet,
        fragment_index: u64,
    ) {
        let mut source_routing_header = packet.routing_header.get_reversed();
        source_routing_header.increase_hop_index();
        if source_routing_header.hop_index != 1 {
            state.logger.log_error(&format!(
                "Unable to reverse source routing header. \n Hops: {} \n Hop index: {}",
                packet.routing_header, packet.routing_header.hop_index
            ));
            return;
        }
        let next_hop = source_routing_header.hops[1];
        let ack = Packet::new_ack(source_routing_header, packet.session_id, fragment_index);

        if let Err(msg) = Self::send_packets_vec(state, &[ack], next_hop) {
            state.logger.log_error(&msg);
            state
                .logger
                .log_debug(&format!("[ACK] Trying to use SC shortcut..."));

            // Send to SC
            if let Err(msg) = Self::sc_send_packet(
                &state.controller_send,
                &DroneEvent::ControllerShortcut(packet.clone()),
            ) {
                state.logger.log_error(&format!("[ACK] - {}", msg));
                state.logger.log_error(&format!(
                    "[ACK] - Unable to forward packet to neither next hop nor SC. \n Packet: {}",
                    packet
                ));
                return;
            }

            state.logger.log_debug(&format!(
                "[ACK] - Successfully sent flood response through SC. Packet: {}",
                packet
            ));
        }
    }
}
