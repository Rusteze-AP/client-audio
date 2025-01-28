mod command_handler;
mod packet_handler;
use super::{Client, Status};
use crossbeam_channel::TryRecvError;
use tokio::time::sleep;
use std::{process::exit, thread};

impl Client {
    #[must_use]
    pub(crate) fn start_message_processing(self) -> thread::JoinHandle<()> {
        let mut init_state = self.state.write().unwrap();
        Self::init_flood_request(&mut init_state);
        drop(init_state);

        thread::spawn(move || loop {
            let mut state = self.state.write().unwrap();
            

            if !state.servers_id.is_empty() && state.status == Status::Idle {
                state.logger.log_info("Server detected, intialize server connection");
                Self::send_subscribe(&mut state);
                Self::send_request_filelist(&mut state);
                state.status = Status::Running;
            }

            if state.status == Status::Terminated {
                break;
            }

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
}
