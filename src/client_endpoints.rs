use crate::Client;
use crossbeam_channel::{unbounded, Receiver, Sender};
use packet_forge::SongMetaData;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;

#[get("/audio/<id>/<segment>")]
pub async fn get_song(
    client: &State<Client>,
    id: String,
    segment: String,
) -> Result<Vec<u8>, NotFound<String>> {
    let state = client.state.clone();

    let (sender, receiver): (Sender<bool>, Receiver<bool>) = unbounded();
    let id: u16 = id.parse().unwrap();
    let mut segment_id = 0;
    if !segment.ends_with(".m3u8") {
        segment_id = segment
            .trim_end_matches(".ts")
            .replace("segment", "")
            .parse::<u16>()
            .unwrap()
            + 1;
    }
    let read_state = state.read().unwrap();
    match read_state.db.get_song_segment(id, segment_id) {
        Ok(payload) => Ok(payload),
        Err(e) => {
            //drop read_state to avoid deadlock
            drop(read_state);

            state
                .read()
                .unwrap()
                .logger
                .log_info(&format!("ask network for segment: db: {}", e));

            // If the segmenent is not found, send request to server
            state
                .write()
                .unwrap()
                .inner_senders
                .insert((id, 0), sender.clone());

            //send request to node
            let mut client_mut = client.inner().clone();
            client_mut.send_segment_request(id, segment_id as u32);

            //waiting for response
            match receiver.recv() {
                Ok(res) => {
                    // check the result
                    if res {
                        let playlist = state
                            .read()
                            .unwrap()
                            .song_map
                            .get(&(id, 0))
                            .unwrap()
                            .clone();
                        Ok(playlist)
                    } else {
                        state
                            .read()
                            .unwrap()
                            .logger
                            .log_error("Song not in the network");
                        Err(NotFound("Song not in the network".to_string()))
                    }
                }
                Err(e) => {
                    state
                        .read()
                        .unwrap()
                        .logger
                        .log_error(&format!("Error on inner channel: {}", e));
                    Err(NotFound(format!("Error on inner channel: {}", e)))
                }
            }
        }
    }
}

#[get("/audio-files")]
pub async fn audio_files(client: &State<Client>) -> Json<Vec<SongMetaData>> {
    let state = client.state.clone();
    let res = state.read().unwrap().db.get_all_songs_meta();
    match res {
        Ok(songs) => Json(songs),
        Err(e) => {
            state
                .read()
                .unwrap()
                .logger
                .log_error(&format!("Error audio_files endpoint: {}", e));
            Json(Vec::new())
        }
    }
}

#[get("/is-ready")]
pub async fn is_ready(client: &State<Client>) -> Json<bool> {
    let state = client.state.clone();
    let res = state.read().unwrap().status == crate::Status::Running;
    Json(res)
}

#[get("/get-id")]
pub async fn get_id(client: &State<Client>) -> Json<u8> {
    let state = client.state.clone();
    let res = state.read().unwrap().id;
    Json(res)
}
