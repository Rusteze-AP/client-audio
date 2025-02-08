use super::{ClientAudio, Status};
use crossbeam_channel::TryRecvError;
use std::thread;
mod command_handler;
mod packet_handler;

impl ClientAudio {
    /// The root function of the message handler thread, it will loop until the status of the client is set to Terminated
    /// It will handle the commands and packets received by the drones or the simulation controller
    pub(crate) fn start_message_processing(self) -> thread::JoinHandle<()> {
        let mut init_state = self.state.write().unwrap();
        Self::init_flood_request(&mut init_state);
        drop(init_state);

        thread::spawn(move || loop {
            let mut state = self.state.write().unwrap();

            // If the client is starting and the server is detected, we initialize the connection with the server
            if !state.servers_id.is_empty() && state.status == Status::Starting {
                state
                    .logger
                    .log_info("Server detected, intialize server connection");
                Self::send_subscribe(&mut state);
                state.status = Status::Idle;
            }

            if state.status == Status::Terminated {
                break;
            }

            // handler for the drone messages
            match state.controller_recv.try_recv() {
                Ok(command) => Self::command_handler(&mut state, command),
                Err(TryRecvError::Empty) => {}
                Err(e) => {
                    state.logger.log_error(&format!(
                        "[{}, {}], error receiving command: {e:?}",
                        file!(),
                        line!()
                    ));
                }
            }

            // handler for the simulation controller commands
            match state.packet_recv.try_recv() {
                Ok(packet) => Self::packet_handler(&mut state, packet),
                Err(TryRecvError::Empty) => {}
                Err(e) => {
                    state.logger.log_error(&format!(
                        "[{}, {}], error receiving packet: {e:?}, ",
                        file!(),
                        line!()
                    ));
                }
            }
        })
    }

    /// Thread that will refresh the network every 60 seconds by sending a request filelist and a flood request.
    pub(crate) fn refresh_network(self) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            let mut state = self.state.write().unwrap();
            if state.status == Status::Terminated {
                break;
            }

            if state.status == Status::Running {
                Self::send_request_filelist(&mut state);
                drop(state);

                thread::sleep(std::time::Duration::from_secs(60));

                let mut state = self.state.write().unwrap();
                Self::init_flood_request(&mut state);
                drop(state);

                thread::sleep(std::time::Duration::from_secs(60));
            }
        })
    }
}
