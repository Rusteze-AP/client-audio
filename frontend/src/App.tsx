import { useState, useEffect, useRef } from "react";
import AudioFileList from "./components/AudioFileList";
import AudioPlayer, { AudioPlayerHandle } from "./components/Player";
import TopBar from "./components/TopBar";

function App() {
  const audioPlayerRef = useRef<AudioPlayerHandle>(null);
  const [isClientReady, setIsClientReady] = useState(false);

  useEffect(() => {
    let interval: any| null = null;

    const checkClientReady = async () => {
      try {
        const response = await fetch("/is-ready");
        const data = await response.json();

        if (data) {
          setIsClientReady(true);

          
          if (interval) {
            clearInterval(interval);
          }
        }
      } catch (error) {
        console.error("Error checking client readiness:", error);
      }
    };

    checkClientReady();
    interval = setInterval(checkClientReady, 1000);

    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, []);

  if (!isClientReady) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-orange-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-orange-500 mx-auto"></div>
          <p className="text-orange-600 mt-4 font-medium">Client in idle state...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-orange-200 min-h-screen flex flex-col">
      <div className="fixed top-0 left-0 w-full z-10 bg-orange-200">
        <TopBar />
      </div>
      <div className="pt-[4rem] pb-[3rem] overflow-y-auto flex-1 px-[5rem] sm:px-[5rem] lg:px-[7rem] xl:px-[10rem]">
        <AudioFileList
          callback={(song) => {
            if (audioPlayerRef.current) {
              audioPlayerRef.current.setSong(song);
            }
          }}
        />
      </div>

      <div className="fixed bottom-0 left-0 w-full bg-orange-300 text-white text-center z-10">
        <AudioPlayer ref={audioPlayerRef} />
      </div>
    </div>
  );
}

export default App;
