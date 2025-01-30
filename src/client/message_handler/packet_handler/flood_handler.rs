use super::ClientAudio;
use crate::ClientState;
use std::sync::RwLockWriteGuard;
use wg_internal::{
    controller::DroneEvent,
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, NodeType, Packet},
};

impl ClientAudio {
    pub(crate) fn init_flood_request(state: &mut RwLockWriteGuard<ClientState>) {
        let flood_req = FloodRequest {
            flood_id: Self::get_flood_id(state),
            initiator_id: state.id,
            path_trace: vec![(state.id, NodeType::Client)],
        };
        let session_id = state.packet_forge.get_session_id();
        let senders = state.senders.clone();
        for (id, sender) in &senders {
            let packet = Packet::new_flood_request(
                SourceRoutingHeader::new(vec![], 0),
                session_id,
                flood_req.clone(),
            );
            if let Err(err) = Self::send_packet(sender, &packet) {
                state
                    .logger
                    .log_error(&format!("[FLOODING] Sending to [DRONE-{}]: {}", id, err));
            }
            let packet_str = Self::get_packet_type(&packet.pack_type);
            Self::event_dispatcher(state, &packet, &packet_str);
        }
    }

    /// Build a flood response for the received flood request
    pub(crate) fn handle_flood_request(
        state: &mut RwLockWriteGuard<ClientState>,
        message: &FloodRequest,
    ) {
        let (dest, packet) = Self::build_flood_response(state.id,message);

        let res = Self::send_flood_response(state, dest, &packet);

        if let Err(msg) = res {
            state.logger.log_error(&msg);
        }
    }

    fn send_flood_response(
        state: &mut RwLockWriteGuard<ClientState>,
        sender: NodeId,
        packet: &Packet,
    ) -> Result<(), String> {

        let sender = Self::get_sender(sender, &state.senders);

        if let Err(err) = sender {
            return Err(format!(
                "[FLOOD RESPONSE] - Error occurred while sending flood response: {}",
                err
            ));
        }

        let sender = sender.unwrap();
        if let Err(err) = Self::send_packet(&sender, packet) {
            state.logger.log_warn(&format!("[FLOOD RESPONSE] - Failed to forward packet to [DRONE-{}]. \n Error: {} \n Trying to use SC shortcut...", packet.routing_header.current_hop().unwrap(), err));
            // Send to SC
            let res = Self::sc_send_packet(
                &state.controller_send,
                &DroneEvent::ControllerShortcut(packet.clone()),
            );

            if let Err(err) = res {
                state
                    .logger
                    .log_error(&format!("[FLOOD RESPONSE] - {}", err));
                return Err(format!(
                    "[FLOOD RESPONSE] - Unable to forward packet to neither next hop nor SC. \n Packet: {}",
                    packet
                ));
            }

            state.logger.log_debug(&format!(
                "[FLOOD RESPONSE] - Successfully sent flood response through SC. Packet: {}",
                packet
            ));
        }
        Ok(())
    }

    pub(crate) fn get_flood_id(state: &mut RwLockWriteGuard<ClientState>) -> u64 {
        state.flood_id += 1;
        state.flood_id
    }

    pub(crate) fn build_flood_response(node_id: NodeId, flood_req: &FloodRequest) -> (NodeId, Packet) {
        let mut flood_req = flood_req.clone();
        flood_req.path_trace.push((node_id, NodeType::Client));
        let mut packet = flood_req.generate_response(1); // Note: returns with hop_index = 0;
        packet.routing_header.increase_hop_index();
        let dest = packet.routing_header.current_hop();

        if dest.is_none() {
            return (0, packet);
        }

        (dest.unwrap(), packet)
    }
}
