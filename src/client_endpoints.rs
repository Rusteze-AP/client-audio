use std::{
    sync::{Arc, RwLock},
    thread::sleep,
};

use base64::{engine::general_purpose, Engine};
use rocket::{
    response::stream::{Event, EventStream},
    serde::json::Json,
    State,
};
use std::time::Duration;

use crate::{database::SongMetaData, ClientState};


#[get("/audio/<id>")]
pub fn stream_audio(state: &State<Arc<RwLock<ClientState>>>, id: String) -> EventStream![] {
    let payload = match state
        .read()
        .unwrap()
        .db
        .get_song_payload(id)
    {
        Ok(p) => p,
        Err(e) => {
            state
                .read()
                .unwrap()
                .logger
                .log_error(&format!("Error: {}", e));
            Vec::new()
        }
    };
    let buffer_size = 1024;
    EventStream! {
        for chunk in payload.chunks(buffer_size) {
            let base64_data = general_purpose::STANDARD.encode(chunk).trim().to_string();

            yield Event::data(base64_data);
            sleep(Duration::from_millis(15));
        }
        yield Event::data(general_purpose::STANDARD.encode("EOF").trim().to_string());
    }
}

#[get("/audio-files")]
pub async fn audio_files(state: &State<Arc<RwLock<ClientState>>>) -> Json<Vec<SongMetaData>> {
    let res = state.read().unwrap().db.get_all_songs_meta();
    match res {
        Ok(songs) => {
            Json(songs)
            },
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
