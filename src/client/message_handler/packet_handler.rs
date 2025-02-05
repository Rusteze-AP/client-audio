use super::ClientAudio;
use crate::ClientState;
use crossbeam_channel::Sender;
use rocket::form::validate::Contains;
use std::{collections::HashMap, sync::RwLockWriteGuard};
use wg_internal::{
    network::NodeId,
    packet::{NodeType, Packet, PacketType},
};
mod ack_handler;
mod flood_handler;
mod fragment_handler;
mod nack_handler;
mod node_messages;

impl ClientAudio {
    pub(crate) fn packet_handler(state: &mut RwLockWriteGuard<ClientState>, packet: Packet) {
        state.routing_handler.nodes_congestion(packet.routing_header.clone());
        match packet.pack_type {
            PacketType::FloodRequest(_) => {}
            _ => {
                if packet.routing_header.hops.last() != Some(&state.id) {
                    state.logger.log_error(&format!(
                        "Received packet for another node {:?} ",
                        packet.routing_header.hops
                    ));
                    return;
                }
            }
        }

        match &packet.pack_type {
            PacketType::MsgFragment(fragment) => {
                Self::fragment_handler(state, fragment, &packet);
            }
            PacketType::FloodResponse(flood_res) => {
                state.logger.log_info(&format!(
                    "Received flood response with flood id: {}",
                    flood_res.flood_id
                ));
                state.routing_handler.update_graph(flood_res.clone());
                for (id, node_type) in &flood_res.path_trace {
                    if *node_type == NodeType::Server && !state.servers_id.contains(id) {
                        state.logger.log_info(&format!("Adding server id: {}", id));
                        state.servers_id.push(*id);
                    }
                }
            }
            PacketType::FloodRequest(flood_req) => {
                Self::handle_flood_request(state, flood_req);
            }
            PacketType::Ack(ack) => {
                Self::ack_handler(state, packet.session_id, ack.fragment_index);
            }
            PacketType::Nack(nack) => {
                Self::nack_handler(state, nack, packet.session_id, packet.routing_header.hops[0]);
            }
        }
    }

    /// Takes a vector of packets and sends them to the `next_hop`
    pub(crate) fn send_packets_vec(
        state: &mut RwLockWriteGuard<ClientState>,
        packets: &[Packet],
        next_hop: NodeId,
    ) -> Result<(), String> {
        // Get the sender channel for the next hop and forward
        let sender = Self::get_sender(next_hop, &state.senders);
        if sender.is_err() {
            return Err(format!("{}", &sender.unwrap_err()));
        }
        let sender = sender.unwrap();

        for packet in packets {
            let packet_str = Self::get_packet_type(&packet.pack_type);
            // IF PACKET IS ACK, NACK OR FLOOD RESPONSE ADD TRY CONTROLLERSHORTCUT
            if packet_str == "Ack" || packet_str == "Nack" || packet_str == "Flood response" {
                // TODO Send Ack Nack Flood response to SC
            }
            if let Err(err) = Self::send_packet(&sender, packet) {
                return Err(format!(
                    "Failed to send packet to [DRONE-{}].\nPacket: {}\n Error: {}",
                    next_hop, packet, err
                ));
            }
            state
                .packets_history
                .insert((packet.get_fragment_index(), packet.session_id), packet.clone());
            Self::event_dispatcher(state, packet, &packet_str);
        }
        Ok(())
    }

    pub(crate) fn send_packet(sender: &Sender<Packet>, packet: &Packet) -> Result<(), String> {
        match sender.send(packet.clone()) {
            Ok(()) => Ok(()),
            Err(err) => Err(format!(
                "Tried sending packet: {packet} but an error occurred: {err}"
            )),
        }
    }

    /// Get the sender channel based on node_id
    pub fn get_sender(
        node_id: NodeId,
        senders: &HashMap<NodeId, Sender<Packet>>,
    ) -> Result<Sender<Packet>, String> {
        if let Some(sender) = senders.get(&node_id) {
            return Ok(sender.clone());
        }
        Err(format!("No neigbour of ID [{node_id}] found."))
    }

    /// Returns the `PacketType` formatted as as `String`
    pub fn get_packet_type(pt: &PacketType) -> String {
        match pt {
            PacketType::Ack(_) => "Ack".to_string(),
            PacketType::Nack(_) => "Nack".to_string(),
            PacketType::FloodRequest(_) => "Flood request".to_string(),
            PacketType::FloodResponse(_) => "Flood response".to_string(),
            PacketType::MsgFragment(_) => "Fragment".to_string(),
        }
    }
}
