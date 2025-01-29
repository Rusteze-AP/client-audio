use packet_forge::{Metadata, SongMetaData};
use sled;
use std::fs;

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
        // Clear the database
        self.db
            .clear()
            .map_err(|e| format!("Error clearing database: {}", e))?;
        self.db
            .flush()
            .map_err(|e| format!("Error flushing database: {}", e))?;

        let dir_entries = fs::read_dir(local_path)
            .map_err(|e| format!("Error reading directory {}: {}", local_path, e))?;

        // Find the JSON file in the directory
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

        // Insert the songs into the database
        for song in songs_array {
            let song: SongMetaData = serde_json::from_value(song.clone())
                .map_err(|e| format!("Error deserializing song: {}", e))?;

            let song_title_parsed = song.title.replace(" ", "").to_lowercase();
            let song_id = self.insert_song_meta(song)?;

            let song_entries = fs::read_dir(format!("{}/songs/{}", local_path, song_title_parsed))
                .map_err(|e| format!("Error reading directory {}: {}", local_path, e))?;

            for entry in song_entries {
                let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
                let path = entry.path();
                if path.is_file() {
                    let entry_content = fs::read(&path).map_err(|e| {
                        format!("Error reading segment file {}: {}", path.display(), e)
                    })?;

                    if path.extension().and_then(|ext| ext.to_str()) == Some("m3u8") {
                        self.insert_song_segment(song_id.clone(), 0,entry_content)?;
                    } else if path.extension().and_then(|ext| ext.to_str()) == Some("ts") {
                        let segment: u32 = path.file_stem().unwrap().to_string_lossy().to_string().replace("segment", "").parse::<u32>().unwrap();
                        self.insert_song_segment(song_id.clone(), segment+1, entry_content)?;
                    } else {
                        return Err("Error: Invalid file extension".to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Insert the song metadata into the database and return the song ID
    /// If the id of the passed song is 0, a new id is generated
    pub fn insert_song_meta(&self, mut song: SongMetaData) -> Result<u16, String> {
        if song.id == 0 {
            song.id = song.compact_hash_u16();
        }

        let serialized_song = bincode::serialize(&song).unwrap();
        match self.db.insert(song.id.to_be_bytes(), serialized_song) {
            Ok(_) => Ok(song.id),
            Err(e) => Err(format!("Error inserting song: {}", e)),
        }
    }

    pub fn insert_song_segment(
        &self,
        id: u16,
        segment: u32,
        payload: Vec<u8>,
    ) -> Result<(), String> {
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(&segment.to_be_bytes());
        key.extend_from_slice(&id.to_be_bytes());
        match self.db.insert(key, payload) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error inserting song payload: {}", e)),
        }
    }

    /// Get the song metadata from the database
    /// If the song is stored locally, the payload is also returned
    pub fn get_song_meta(&self, id: u16) -> Result<SongMetaData, String> {
        match self.db.get(id.to_be_bytes()) {
            Ok(Some(data)) => {
                let song: SongMetaData = bincode::deserialize(&data).unwrap();
                Ok(song)
            }
            Ok(None) => Err("Song not found".to_string()),
            Err(e) => Err(format!("Error getting song: {}", e)),
        }
    }

    pub fn get_song_segment(&self, id: u16, segment: u32) -> Result<Vec<u8>, String> {
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(&segment.to_be_bytes());
        key.extend_from_slice(&id.to_be_bytes());
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
                    if *key.get(0).unwrap() != 0 {
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
