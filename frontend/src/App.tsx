import { useRef } from "react";
import AudioFileList from "./components/AudioFileList";
import AudioPlayer, { AudioPlayerHandle } from "./components/Player";
import TopBar from "./components/TopBar";

function App() {
  const audioPlayerRef = useRef<AudioPlayerHandle>(null);

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
