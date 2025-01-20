
const TopBar = () => {
  return (
    <div className="flex justify-between items-center px-6 py-4">
      <div className="flex items-center">
        <span className="text-xl font-bold">Rust-eze</span>
      </div>

      <div className="text-xl text-gray-800">
        Ruste-eze Audio Streaming
      </div>

      <div className="text-sm text-gray-600">
        Client ID: 20
      </div>
    </div>
  );
};

export default TopBar;
