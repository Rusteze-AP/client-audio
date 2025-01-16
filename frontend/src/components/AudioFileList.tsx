import { useEffect, useState } from "react";

type AudioFileListProps = {
  callback: (file: number) => void;
};

interface Song {
  id: number;
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
      setSongs(files);
    } catch (error) {
      console.error("Error on fetching files:", error);
    }
  };

  useEffect(() => {
    fetchAudioFiles();
  }, []);

  return (
    <div className="grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-3 text-left">
      {songs.map((song, _) => (
        <div onClick={() => callback(song.id)} className="group relative overflow-hidden rounded-2xl">
          <img
            src={song.image_url}
            alt={song.title}
            className="w-[25vw] h-auto object-cover transition-transform duration-300 ease-in-out transform group-hover:scale-110 "
          />
          <div className="absolute inset-0 bg-black bg-opacity-50 opacity-0 transition-opacity duration-300 group-hover:opacity-100 flex flex-col justify-between p-4">
            <div className="flex justify-end">
              <span className="text-white text-lg font-semibold px-2 py-1 rounded-md">{formatDuration(song.duration)}</span>
            </div>

            <div className="text-white">
              <h3 className="text-3xl font-bold">{song.title}</h3>
              <p className="text-lg text-gray-300">
                {song.album} • {song.artist}
              </p>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

const formatDuration = (duration: number) => {
  const minutes = Math.floor(duration / 60);
  const seconds = duration % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
};

export default AudioFileList;
