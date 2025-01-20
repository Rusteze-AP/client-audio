use std::fs;

use serde::{Deserialize, Serialize};
use sled;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)] 
pub struct SongMetaData {
    id: String,
    title: String,
    artist: String,
    album: String,
    duration: u8,
    image_url: String,
    is_local: bool,
}

pub struct AudioDatabase {
    db: sled::Db,
}

impl AudioDatabase {
    pub fn new(database: &str) -> Self {
        let db = match sled::open(database) {
            Ok(db) => AudioDatabase { db },
            Err(e) => {
                eprintln!("Error opening database: {}", e);
                std::process::exit(1);
            }
        };

        db
    }

    /// Initialize the database by clearing it and inserting local files
    pub fn init(&self, local_path: &str) -> Result<(), String> {
        self.db
            .clear()
            .map_err(|e| format!("Error clearing database: {}", e))?;
        self.db
            .flush()
            .map_err(|e| format!("Error flushing database: {}", e))?;

        let dir_entries = fs::read_dir(local_path)
            .map_err(|e| format!("Error reading directory {}: {}", local_path, e))?;

        let mut json_file_path = None;
        for entry in dir_entries {
            let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if json_file_path.is_some() {
                    return Err(
                        "Error: More than one .json file found in the directory".to_string()
                    );
                }
                json_file_path = Some(path);
            }
        }

        let json_file_path = json_file_path
            .ok_or_else(|| format!("Error: No .json file found in directory {}", local_path))?;

        let file_content = fs::read_to_string(&json_file_path).map_err(|e| {
            format!(
                "Error reading metadata file {}: {}",
                json_file_path.display(),
                e
            )
        })?;

        let json_data: serde_json::Value = serde_json::from_str(&file_content)
            .map_err(|e| format!("Error parsing JSON metadata: {}", e))?;

        let songs_array = json_data["songs"]
            .as_array()
            .ok_or_else(|| "Error JSON metadata is not valid".to_string())?;

        for song in songs_array {
            let song: SongMetaData = serde_json::from_value(song.clone())
                .map_err(|e| format!("Error deserializing song: {}", e))?;

            let song_title_parsed = song.title.replace(" ", "").to_lowercase();
            let mp3_file_path = format!("{}/songs/{}.mp3", local_path, song_title_parsed);

            let mp3_content = fs::read(&mp3_file_path)
                .map_err(|e| format!("Error reading MP3 file {}: {}", mp3_file_path, e))?;
            let song_id = self.insert_song_meta(song)?;
            self.insert_song_payload(song_id, mp3_content)?;
        }
        Ok(())
    }

    /// Insert the song metadata into the database and return the song ID
    /// If the id of the passed song is 0, a new id is generated
    pub fn insert_song_meta(&self, mut song: SongMetaData) -> Result<String, String> {
        if song.id == "0" {
            song.id = Uuid::new_v4().to_string();
        }

        let serialized_song = bincode::serialize(&song).unwrap();
        match self.db.insert(song.id.as_bytes(), serialized_song) {
            Ok(_) => Ok(song.id.clone()),
            Err(e) => Err(format!("Error inserting song: {}", e)),
        }
    }

    pub fn insert_song_payload(&self, id: String, payload: Vec<u8>) -> Result<(), String> {
        let mut key: Vec<u8> = Vec::new();
        key.push(1);
        key.extend_from_slice(id.as_bytes());
        match self.db.insert(key, payload) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error inserting song payload: {}", e)),
        }
    }

    /// Get the song metadata from the database
    /// If the song is stored locally, the payload is also returned
    pub fn get_song_meta(&self, id: String) -> Result<SongMetaData, String> {
        match self.db.get(id.as_bytes()) {
            Ok(Some(data)) => {
                let song: SongMetaData = bincode::deserialize(&data).unwrap();
                Ok(song)
            }
            Ok(None) => Err("Song not found".to_string()),
            Err(e) => Err(format!("Error getting song: {}", e)),
        }
    }

    /// Get the song payload from the database
    pub fn get_song_payload(&self, id: String) -> Result<Vec<u8>, String> {
        let mut key: Vec<u8> = Vec::new();
        key.push(1);
        key.extend_from_slice(id.as_bytes());
        match self.db.get(key) {
            Ok(Some(data)) => Ok(data.to_vec()),
            Ok(None) => Err("Song payload not found".to_string()),
            Err(e) => Err(format!("Error getting song payload: {}", e)),
        }
    }

    /// Get all the songs metadata from the database
    pub fn get_all_songs_meta(&self) -> Result<Vec<SongMetaData>, String> {
        let mut songs = Vec::new();

        for record in self.db.iter() {
            match record {
                Ok((key, data)) => {
                    if *key.get(0).unwrap() != 1 {
                        match bincode::deserialize(&data) {
                            Ok(song) => songs.push(song),
                            Err(e) => return Err(format!("Error deserializing song: {}", e)),
                        }
                    }
                }
                Err(e) => return Err(format!("Error iterating database: {}", e)),
            }
        }
        Ok(songs)
    }

}