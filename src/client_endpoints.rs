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

// #[get("/audio/<file>")]
// pub fn stream_audio(file: String) -> EventStream![] {
//     EventStream! {
//         let file_path = format!("/home/carne/projects/rusteze/client-audio/static/audio/{}", file);
//         let mut file = File::open(file_path).expect("File non trovato");
//         let mut buffer = [0; 128];  // 1 KB buffer per chunk
//         let mut i = 0;
//         while let Ok(n) = file.read(&mut buffer) {
//             if n == 0 {
//                 yield Event::data(general_purpose::STANDARD.encode("EOF").trim().to_string());
//                 break;
//             }

//             let base64_data = general_purpose::STANDARD.encode(&buffer[..n]).trim().to_string();

//             println!("Chunk {}, size {}", i,base64_data.len());
//             i += 1;
//             yield Event::data(base64_data);
//             sleep(Duration::from_millis(1));
//         }
//     }
// }

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
            sleep(Duration::from_millis(1));
        }
        yield Event::data(general_purpose::STANDARD.encode("EOF").trim().to_string());
    }
}

#[get("/audio-files")]
pub async fn audio_files(state: &State<Arc<RwLock<ClientState>>>) -> Json<Vec<SongMetaData>> {
    let res = state.read().unwrap().db.get_all_songs_meta();
    match res {
        Ok(songs) => {
            println!("Songs: {:?}", songs);
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
