interface VolumeBarProps {
  volume: number;
  onVolumeChange: (volume: number) => void;
}

const VolumeBar = ({ volume, onVolumeChange }: VolumeBarProps) => {

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onVolumeChange(Number(e.target.value));
  };

  const toggleMute = () => {
    if (volume == 0 ){
        onVolumeChange(100);
    }
    else{
        onVolumeChange(0);
    }
  }

  return (
    <div className="flex items-center justify-start space-x-2">
      <i className={`fas ${volume == 0 ? "fa-volume-xmark" : volume > 50 ? "fa-volume-high" : "fa-volume-low" } text-sm text-black cursor-pointer min-w-6`} onClick={toggleMute}></i>
      <input
        type="range"
        min="0"
        max="100"
        value={volume}
        onChange={handleVolumeChange}
        className="progress-bar"
        style={{
          background: `linear-gradient(to right, black ${volume}%, gray ${volume}%)`,
        }}
      />
    </div>
  );
};

export default VolumeBar;
