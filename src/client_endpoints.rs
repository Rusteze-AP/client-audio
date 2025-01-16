use std::{
    fs::{self, File},
    io::Read,
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

#[get("/audio/<file>")]
pub fn stream_audio(file: String) -> EventStream![] {
    EventStream! {
        let file_path = format!("/home/carne/projects/rusteze/client-audio/static/audio/{}", file);
        let mut file = File::open(file_path).expect("File non trovato");
        let mut buffer = [0; 128];  // 1 KB buffer per chunk
        let mut i = 0;
        while let Ok(n) = file.read(&mut buffer) {
            if n == 0 {
                yield Event::data(general_purpose::STANDARD.encode("EOF").trim().to_string());
                break;
            }

            let base64_data = general_purpose::STANDARD.encode(&buffer[..n]).trim().to_string();

            println!("Chunk {}, size {}", i,base64_data.len());
            i += 1;
            yield Event::data(base64_data);
            sleep(Duration::from_millis(1));
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
