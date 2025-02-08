use super::ClientAudio;
use crate::{ClientState, Status};
use crossbeam_channel::Sender;
use std::sync::RwLockWriteGuard;
use wg_internal::{controller::{DroneCommand, DroneEvent}, network::NodeId, packet::Packet};

impl ClientAudio {
    /// Handles the command received from the simulation controller.
    pub(crate) fn command_handler(
        state: &mut RwLockWriteGuard<ClientState>,
        command: DroneCommand,
    ) {
        if state.status != Status::Terminated {
            let res = match command {
                DroneCommand::RemoveSender(id) => Self::remove_sender(state, id),
                DroneCommand::AddSender(id, sender) => Self::add_sender(state, id, &sender),
                // when the simulation controller sends a crash command, change the status to terminated
                DroneCommand::Crash => {
                    state
                        .logger
                        .log_debug("[SC COMMAND]]Received crash command. Terminating!");
                    state.status = Status::Terminated;
                    Ok(())
                }
                _ => Err("[SC COMMAND]Received unhandled SC command (ChangePdr)!".to_string()),
            };

            if let Err(err) = res {
                state.logger.log_error(&err);
            }
        }
    }

    /// Sends a `DroneEvent` containing the `packet` that has been sent.
    pub(crate) fn event_dispatcher(
        state: &mut RwLockWriteGuard<ClientState>,
        packet: &Packet,
        packet_str: &str,
    ) {
        if let Err(err) = Self::sc_send_packet(
            &state.controller_send,
            &DroneEvent::PacketSent(packet.clone()),
        ) {
            state.logger.log_error(&format!(
                "[{}] - Packet event forward: {}",
                packet_str.to_ascii_uppercase(),
                err
            ));
            return;
        }
        state.logger.log_debug(&format!(
            "[{}] - Packet event sent successfully",
            packet_str.to_ascii_uppercase()
        ));
    }

    /// Sends a command to the simulation controller.
    pub(crate) fn sc_send_packet(
        sender: &Sender<DroneEvent>,
        packet: &DroneEvent,
    ) -> Result<(), String> {
        match sender.send(packet.clone()) {
            Ok(()) => Ok(()),
            Err(err) => Err(format!(
                "Error occurred while sending packet event to SC. Error: {err}"
            )),
        }
    }

    /// Remove the sender with the given `id` from the `senders` map.
    pub(crate) fn remove_sender(
        state: &mut RwLockWriteGuard<ClientState>,
        id: NodeId,
    ) -> Result<(), String> {
        let res = state.senders.remove(&id);
        if res.is_none() {
            return Err(format!("[REMOVE SENDER] - Sender with id {} not found", id));
        }
        state
            .logger
            .log_debug(&format!("[REMOVE SENDER] - Sender with id {} removed", id));
        Ok(())
    }

    /// Add the sender with the given `id` to the `senders` map.
    pub(crate) fn add_sender(
        state: &mut RwLockWriteGuard<ClientState>,
        id: NodeId,
        sender: &Sender<Packet>,
    ) -> Result<(), String> {
        let res = state.senders.insert(id, sender.clone());
        if res.is_some() {
            return Err(format!(
                "[ADD SENDER] - Sender with id {} already exists",
                id
            ));
        }
        state
            .logger
            .log_debug(&format!("[ADD SENDER] - Sender with id {} added", id));
        Ok(())
    }
}
