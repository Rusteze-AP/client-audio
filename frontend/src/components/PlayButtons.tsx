
interface PlayButtonsProps {
  isStreaming: boolean;
  onToggle: () => void;
}

const PlayButtons: React.FC<PlayButtonsProps> = ({ isStreaming, onToggle }) => {


  return (
    <div className="flex items-center justify-center space-x-2">
      <div onClick={()=>onToggle()} className="flex items-center justify-center w-9 h-14 cursor-pointer text-black hover:text-gray-700 transition">
        <i className={`fas ${isStreaming ? "fa-pause" : "fa-play"} text-xl`}></i>
      </div>
    </div>
  );
};

export default PlayButtons;
