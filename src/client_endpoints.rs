use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::{Arc, RwLock};

use crate::{database::SongMetaData, ClientState};

#[get("/audio/<id>/<segment>")]
pub async fn get_song(
    state: &State<Arc<RwLock<ClientState>>>,
    id: String,
    segment: String,
) -> Result<Vec<u8>, NotFound<String>> {
    if segment.ends_with(".m3u8") {
        let playlist = state.read().unwrap().db.get_song_playlist(id.clone());

        match playlist {
            Ok(playlist) => Ok(playlist),
            Err(e) => {
                state
                    .read()
                    .unwrap()
                    .logger
                    .log_error(&format!("Error get_song playlist: {}", e));
                Err(NotFound(format!("Error get_song playlist: {}", e)))
            }
        }
    } else {
        let segment_id = segment.trim_end_matches(".ts");
        let segment_payload = state
            .read()
            .unwrap()
            .db
            .get_song_segment(id.clone(), segment_id.to_string());
        match segment_payload {
            Ok(s) => Ok(s),
            Err(e) => {
                state
                    .read()
                    .unwrap()
                    .logger
                    .log_error(&format!("Error get_song segment: {}", e));
                Err(NotFound(format!("Error get_song segment: {}", e)))
            }
        }
    }
}

#[get("/audio-files")]
pub async fn audio_files(state: &State<Arc<RwLock<ClientState>>>) -> Json<Vec<SongMetaData>> {
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
