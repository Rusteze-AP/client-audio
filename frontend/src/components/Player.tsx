import { forwardRef, useRef, useState, useEffect, useImperativeHandle } from "react";
import Hls from "hls.js";
import PlayButtons from "./PlayButtons";
import TimeBar from "./TimeBar";
import VolumeBar from "./VolumeBar";
import { Song } from "./AudioFileList";

export type AudioPlayerHandle = {
  setSong: (song: Song) => void;
};

const AudioPlayer = forwardRef((_props, ref) => {
  const audioRef = useRef<HTMLAudioElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const currentSongRef = useRef<Song | null>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [bufferedTime, setBufferedTime] = useState(0);
  const [volume, setVolume] = useState(100);

  useImperativeHandle(ref, () => ({
    setSong,
  }));

  const togglePlay = () => {
    if (audioRef.current) {
      if (isPlaying) {
        audioRef.current.pause();
      } else {
        audioRef.current.play().catch(console.error);
      }
      setIsPlaying(!isPlaying);
    }
  };

  const onVolumeChange = (volume: number) => {
    if (audioRef.current) {
      audioRef.current.volume = volume / 100;
      setVolume(volume);
    }
  };

  useEffect(() => {
    const audioElement = audioRef.current;

    const onTimeUpdate = () => {
      if (audioRef.current) {
        setCurrentTime(Math.ceil(audioRef.current.currentTime));
        
        // Aggiorna il tempo bufferizzato
        const buffered = audioRef.current.buffered;
        if (buffered.length > 0) {
          setBufferedTime(Math.round(buffered.end(buffered.length - 1)));
        }
      }
    };

    const onEnded = () => {
      setIsPlaying(false);
      if (currentSongRef.current) {
        setCurrentTime(currentSongRef.current.duration);
      }
    };

    if (audioElement) {
      audioElement.addEventListener("timeupdate", onTimeUpdate);
      audioElement.addEventListener("ended", onEnded);
    }

    return () => {
      if (audioElement) {
        audioElement.removeEventListener("timeupdate", onTimeUpdate);
        audioElement.removeEventListener("ended", onEnded);
      }
    };
  }, []);

  const setSong = (song: Song) => {
    stopPlayback();
    currentSongRef.current = song;
    setBufferedTime(0);
    setCurrentTime(0);
    startPlayback(song.id);
  };

  const onTimeChange = (time: number) => {
    if (audioRef.current) {
      audioRef.current.currentTime = time;
    }
  };

  const startPlayback = (id: string) => {
    if (!audioRef.current) return;

    const streamUrl = `/audio/${id}/playlist.m3u8`;

    if (Hls.isSupported()) {
      if (hlsRef.current) {
        hlsRef.current.destroy();
      }

      const hls = new Hls({
        enableWorker: true,
        lowLatencyMode: true,
      });
      
      hlsRef.current = hls;
      hls.loadSource(streamUrl);
      hls.attachMedia(audioRef.current);

      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        audioRef.current?.play()
          .then(() => setIsPlaying(true))
          .catch(console.error);
      });

      hls.on(Hls.Events.ERROR, (_, data) => {
        if (data.fatal) {
          switch (data.type) {
            case Hls.ErrorTypes.NETWORK_ERROR:
              console.error("Network error, trying to recover...");
              hls.startLoad();
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              console.error("Media error, trying to recover...");
              hls.recoverMediaError();
              break;
            default:
              console.error("Fatal error, stopping playback");
              stopPlayback();
              break;
          }
        }
      });
    } else if (audioRef.current.canPlayType('application/vnd.apple.mpegurl')) {
      // Per Safari che ha supporto HLS nativo
      audioRef.current.src = streamUrl;
      audioRef.current.play()
        .then(() => setIsPlaying(true))
        .catch(console.error);
    }
  };

  const stopPlayback = () => {
    setIsPlaying(false);
    setBufferedTime(0);
    setCurrentTime(0);

    if (hlsRef.current) {
      hlsRef.current.destroy();
      hlsRef.current = null;
    }

    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.removeAttribute("src");
      audioRef.current.load();
    }
  };

  return (
    <div className="flex justify-between items-center px-[5rem] sm:px-[5rem] lg:px-[7rem] xl:px-[10rem]">
      <div className="flex items-center justify-center">
        <PlayButtons
          isStreaming={isPlaying}
          onToggle={togglePlay}
          onSkip={() => {
            console.log("skippp");
          }}
        />
      </div>

      <div className="flex-grow mx-5 flex items-center justify-center">
        <TimeBar
          currentTime={currentTime}
          bufferedTime={bufferedTime}
          duration={currentSongRef.current ? currentSongRef.current.duration : 0}
          onTimeChange={onTimeChange}
        />
      </div>

      <div className="flex items-center min-w-40 max-w-40">
        <VolumeBar volume={volume} onVolumeChange={onVolumeChange} />
      </div>
      <audio ref={audioRef} />
    </div>
  );
});

export default AudioPlayer;