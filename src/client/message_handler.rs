mod command_handler;
mod packet_handler;
use super::Client;
use crossbeam_channel::TryRecvError;
use std::thread;

impl Client {
    #[must_use]
    pub(crate) fn start_message_processing(self) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            let mut state = self.state.write().unwrap();

            if state.terminated {
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
