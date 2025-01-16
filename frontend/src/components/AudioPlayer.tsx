import { useRef, useState, useEffect } from "react";
import AudioFileList from "./AudioFileList";

function AudioPlayer() {
  const audioRef = useRef<HTMLAudioElement>(null);
  const eventSourceRef = useRef<EventSource | null>(null);
  const mediaSourceRef = useRef<MediaSource | null>(null);
  const sourceBufferRef = useRef<SourceBuffer | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);

  const logCurrentTime = () => {
    if (audioRef.current) {
      // console.log(`current time: ${audioRef.current.currentTime}`);
    }
  };

  useEffect(() => {
    const audioElement = audioRef.current;

    const onTimeUpdate = () => {
      logCurrentTime();
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

  const startStream = (file: number) => {
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

    eventSourceRef.current = new EventSource(`http://localhost:8000/audio/${file}`);

    eventSourceRef.current.onmessage = (event) => {
      if (!sourceBufferRef.current || sourceBufferRef.current.updating) {
        return;
      }

      const base64Data = event.data;
      const binaryString = atob(base64Data);
      if (binaryString === "EOF") {
        console.log("End of stream");
        if (eventSourceRef.current) {
          eventSourceRef.current.close();
          eventSourceRef.current = null;
        }
        return;
      }
      const buffer = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        buffer[i] = binaryString.charCodeAt(i);
      }

      try {
        sourceBufferRef.current.appendBuffer(buffer);
      } catch (appendErr) {
        console.error("appendBuffer error:", appendErr);
      }
    };

    eventSourceRef.current.onerror = (err) => {
      console.error("SSE error:", err);
      stopStream();
    };
  };

  const stopStream = () => {
    setIsStreaming(false);

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
    <div style={{ padding: "1rem" }}>
      <audio ref={audioRef} controls />
      <AudioFileList callback={startStream} />
    </div>
  );
}

export default AudioPlayer;
