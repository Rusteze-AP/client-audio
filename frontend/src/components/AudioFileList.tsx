import { useEffect, useState } from "react";
import { formatTime } from "../tools";

type AudioFileListProps = {
  callback: (song: Song) => void;
};

export interface Song {
  id: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  image_url: string;
}

const AudioFileList: React.FC<AudioFileListProps> = ({ callback }) => {
  const [songs, setSongs] = useState<Song[]>([]);

  const fetchAudioFiles = async () => {
    try {
      const response = await fetch("/audio-files");
      if (!response.ok) {
        throw new Error(`Erroron fetch: ${response.statusText}`);
      }
      const files = await response.json();
      console.log("Files:", files);
      setSongs(files);
    } catch (error) {
      console.error("Error on fetching files:", error);
    }
  };

  useEffect(() => {
    fetchAudioFiles();
  }, []);

  return (
    <div className="grid grid-cols-3 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-4 p-4">
      {songs.map((song) => (
        <div key={song.id} onClick={() => callback(song)} className="group relative w-full aspect-square overflow-hidden rounded-2xl bg-gray-800">
          <img
            src={song.image_url}
            alt={song.title}
            className="object-cover w-full h-full transition-transform duration-300 ease-in-out transform group-hover:scale-110"
          />
          <div className="absolute inset-0 bg-black bg-opacity-50 opacity-0 transition-opacity duration-300 group-hover:opacity-100 flex flex-col justify-between p-4">
            <div className="flex justify-end">
              <span className="text-white text-lg font-semibold px-2 py-1 rounded-md select-none">{formatTime(song.duration)}</span>
            </div>
            <div className="text-white select-none">
              <h3 className="text-xl font-bold">{song.title}</h3>
              <p className="text-sm text-gray-300">
                {song.album} • {song.artist}
              </p>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

export default AudioFileList;
