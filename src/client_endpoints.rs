use crate::ClientAudio;
use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use packet_forge::SongMetaData;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use std::time::Duration;
use rocket::State;

/// Get the song payload from the network
/// 
/// It first checks if the song is in the database.
/// If it is not in the database, it sends a peer list request to the server and waits for the response.
/// The server answers with the node that has the song and than the client sends a request to the node.
/// 
/// When the request is sent the thread waits to receive the response from the other thread with a crossbeam channel.
#[get("/audio/<id>/<segment>")]
pub async fn get_song(
    client: &State<ClientAudio>,
    id: &str,
    segment: &str,
) -> Result<Vec<u8>, NotFound<String>> {
    let state = client.state.clone();

    let (sender, receiver): (Sender<bool>, Receiver<bool>) = unbounded();
    let id: u16 = id.parse().unwrap();
    let mut segment_id: u32 = 0;
    if !segment.ends_with(".m3u8") {
        segment_id = segment
            .trim_end_matches(".ts")
            .replace("segment", "")
            .parse::<u32>()
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
                .insert((id, segment_id), sender.clone());

            //send request to node
            let mut client_mut = client.inner().clone();
            client_mut.send_segment_request(id, segment_id as u32);

            //waiting for response from the other thread
            match receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(res) => {
                    // check the result
                    if res {
                        let song_map = state.read().unwrap().song_map.clone();

                        let playlist = song_map
                            .get(&(id, segment_id))
                            .unwrap()
                            .clone();  

                        // remove the song from the map as it is cached in the frontend
                        state.write().unwrap().song_map.remove(&(id, segment_id));

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
                Err(RecvTimeoutError::Timeout) => {
                    state
                        .read()
                        .unwrap()
                        .logger
                        .log_error("Timeout while waiting for song");
                    Err(NotFound("Timeout while waiting for song".to_string()))
                }
                Err(RecvTimeoutError::Disconnected) => {
                    state
                        .read()
                        .unwrap()
                        .logger
                        .log_error("Channel disconnected");
                    Err(NotFound("Channel disconnected".to_string()))
                }
            }
        }
    }
}

/// Get the song metadata that is syncronized with the network
#[get("/audio-files")]
pub async fn audio_files(client: &State<ClientAudio>) -> Json<Vec<SongMetaData>> {
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

/// Check if the client is ready to stream audio
#[get("/is-ready")]
pub async fn is_ready(client: &State<ClientAudio>) -> Json<bool> {
    let state = client.state.clone();
    println!("is_ready???? {:?}", state.read().unwrap().status);
    let res = state.read().unwrap().status == crate::Status::Running;
    print!("is_ready???? {:?}", res);
    Json(res)
}

/// Get the client id
#[get("/get-id")]
pub async fn get_id(client: &State<ClientAudio>) -> Json<u8> {
    let state = client.state.clone();
    let res = state.read().unwrap().id;
    Json(res)
}
