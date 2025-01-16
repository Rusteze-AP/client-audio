import AudioFileList from "./components/AudioFileList";

function App() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100 p-4">
      {/* <AudioPlayer/> */}
      <AudioFileList callback={(file)=>{console.log("file {}",file)}}/>
    </div>
  );
}

export default App;
