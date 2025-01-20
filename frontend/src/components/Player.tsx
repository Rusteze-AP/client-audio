import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import VolumeBar from "./VolumeBar";
import PlayButtons from "./PlayButtons";
import TimeBar from "./TimeBar";
import { Song } from "./AudioFileList";

export type AudioPlayerHandle = {
  setSong: (song: Song) => void;
};

const AudioPlayer = forwardRef((_props, ref) => {
  const audioRef = useRef<HTMLAudioElement>(null);
  const eventSourceRef = useRef<EventSource | null>(null);
  const mediaSourceRef = useRef<MediaSource | null>(null);
  const sourceBufferRef = useRef<SourceBuffer | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const currentSongRef = useRef<Song | null>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [bufferedTime, setBufferedTime] = useState(0);
  const [volume, setVolume] = useState(100);

  useImperativeHandle(ref, () => ({
    setSong,
  }));

  const togglePlay = () => {
    if (!isStreaming && currentSongRef.current !== null) {
      if (audioRef.current) {
        audioRef.current.play().catch((err) => {
          console.error("Error on .play()", err);
        });
        setIsStreaming(true);
      }
    } else {
      if (audioRef.current) {
        audioRef.current.pause();
        setIsStreaming(false);
      }
    }
  };

  const onVolumeChange = (volume: number) => {
    if (audioRef.current) {
      audioRef.current.volume = volume / 100;
      setVolume(volume);
    }
  };

  useEffect(() => {
    if (audioRef.current) {
      if (currentSongRef.current !== null && currentTime >= currentSongRef.current.duration) {
        console.log("Song ended");
        stopStream();
      }
    }
  }, [currentTime]);

  useEffect(() => {
    const audioElement = audioRef.current;

    const onTimeUpdate = () => {
      if (audioRef.current) {
        setCurrentTime(Math.ceil(audioRef.current.currentTime));
      }
    };

    if (audioElement) {
      audioElement.addEventListener("timeupdate", onTimeUpdate);
    }

    return () => {
      if (audioElement) {
        audioElement.removeEventListener("timeupdate", onTimeUpdate);
      }
    };
  }, []);

  const setSong = (song: Song) => {
    stopStream();
    currentSongRef.current = song;
    setBufferedTime(0);
    setCurrentTime(0);
    startStream(song.id);
  };

  const onTimeChange = (time: number) => {
    if (audioRef.current) {
      if (time >= bufferedTime) {
        audioRef.current.currentTime = currentSongRef.current!.duration;
      } else {
        audioRef.current.currentTime = time;
      }
    }
  };

  const startStream = (id: string) => {
    if (isStreaming) stopStream();
    setIsStreaming(true);

    const mediaSource = new MediaSource();
    mediaSourceRef.current = mediaSource;

    const objectURL = URL.createObjectURL(mediaSource);
    if (audioRef.current) {
      audioRef.current.src = objectURL;
      audioRef.current.play().catch((err) => {
        console.error("Error on .play()", err);
      });
    }

    mediaSource.addEventListener("sourceopen", () => {
      const mimeCodec = "audio/mpeg";
      try {
        if (mediaSourceRef.current) {
          sourceBufferRef.current = mediaSourceRef.current.addSourceBuffer(mimeCodec);
        } else {
          console.error("mediaSourceRef.current is null");
        }
      } catch (e) {
        console.error("Error addSourceBuffer:", e);
        return;
      }
    });

    eventSourceRef.current = new EventSource(`/audio/${id}`);

    eventSourceRef.current.onmessage = (event) => {
      if (!sourceBufferRef.current || sourceBufferRef.current.updating) {
        return;
      }

      const base64Data = event.data;
      const binaryString = atob(base64Data);
      if (binaryString === "EOF") {
        if (eventSourceRef.current) {
          eventSourceRef.current.close();
          eventSourceRef.current = null;
        }
        if (mediaSourceRef.current && mediaSourceRef.current.readyState === "open") {
          try {
            mediaSourceRef.current.endOfStream();
          } catch (e) {
            console.warn("Error ending MediaSource stream:", e);
          }
        }
        if (currentSongRef.current) {
          setTimeout(() => {
            setBufferedTime(currentSongRef.current!.duration);
          }, 10);
        } else {
          console.error("currentSongRef is null, unable to set bufferedTime");
        }
        return;
      }
      const buffer = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        buffer[i] = binaryString.charCodeAt(i);
      }

      try {
        sourceBufferRef.current.appendBuffer(buffer);

        if (audioRef.current && sourceBufferRef.current) {
          const buffered = audioRef.current.buffered;
          if (buffered.length > 0) {
            setTimeout(() => {
              setBufferedTime(Math.round(buffered.end(buffered.length - 1)));
            }, 10);
          }
        }
      } catch (appendErr) {
        console.error("appendBuffer error:", appendErr);
        stopStream();
      }
    };

    eventSourceRef.current.onerror = (err) => {
      console.error("SSE error:", err);
      stopStream();
    };
  };

  const stopStream = () => {
    setIsStreaming(false);
    setBufferedTime(0);
    setCurrentTime(0);

    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.removeAttribute("src");
      audioRef.current.load();
    }

    if (mediaSourceRef.current) {
      if (mediaSourceRef.current.readyState === "open") {
        try {
          mediaSourceRef.current.endOfStream();
        } catch (e) {
          console.warn("Impossible endOfStream:", e);
        }
      }
      mediaSourceRef.current = null;
    }

    if (sourceBufferRef.current) {
      sourceBufferRef.current = null;
    }
  };

  return (
    <div className="flex justify-between items-center px-[5rem] sm:px-[5rem] lg:px-[7rem] xl:px-[10rem]">
      <div className="flex items-center justify-center">
        <PlayButtons
          isStreaming={isStreaming}
          onToggle={() => togglePlay()}
          onSkip={() => {
            console.log("skippp");
          }}
        />
      </div>

      {/* time bar */}
      <div className="flex-grow mx-5 flex items-center justify-center">
        <TimeBar
          currentTime={currentTime}
          bufferedTime={bufferedTime}
          duration={currentSongRef.current ? currentSongRef.current.duration : 0}
          onTimeChange={onTimeChange}
        />
      </div>

      {/* Volume bar */}
      <div className="flex items-center min-w-40 max-w-40">
        <VolumeBar volume={volume} onVolumeChange={onVolumeChange} />
      </div>
      <audio ref={audioRef} />
    </div>
  );
});

export default AudioPlayer;
