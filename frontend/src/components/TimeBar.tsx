import { useEffect } from "react";
import { formatTime } from "../tools";

interface TimeBarProps {
  duration: number;
  bufferedTime: number;
  currentTime: number;
  onTimeChange: (time: number) => void;
}

const TimeBar = ({ duration, currentTime, bufferedTime, onTimeChange }: TimeBarProps) => {

  const currentTimePercentage = currentTime === 0 ? 0 : (currentTime / duration) * 100;
  const bufferedTimePercentage = bufferedTime === 0 ? 0 : (bufferedTime / duration) * 100;


  return (
    <div className="flex justify-between items-center w-full space-x-2">
      <p className="text-sm text-black">{formatTime(currentTime)}</p>
      <input
        type="range"
        min="0"
        max={duration}
        value={currentTime}
        onChange={(e) => onTimeChange(Number(e.target.value))}
        className="progress-bar w-full"
        style={{
          background: `
            linear-gradient(
              to right, 
              black ${currentTimePercentage}%, 
              darkgray ${currentTimePercentage}% ${bufferedTimePercentage}%, 
              lightgray ${bufferedTimePercentage}%
            )
          `,
        }}
      />
      <p className="text-sm text-black">{formatTime(duration)}</p>
    </div>
  );
};

export default TimeBar;
