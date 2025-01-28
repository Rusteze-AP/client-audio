import { useEffect, useState } from 'react';
  
const TopBar = () => {
  const [clientId, setClientId] = useState<number | null>(null);

  const fetchClientId = async () => {
    try {
      const response = await fetch("/get-id");
      if (!response.ok) {
        throw new Error(`Erroron fetch: ${response.statusText}`);
      }
      const id = await response.json();
      console.log('Client ID:', id);
      setClientId(id);
    } catch (error) {
      console.error('Error fetching client ID:', error);
    }
  };

  useEffect(() => {
    fetchClientId();
  }, []);

  return (
    <div className="flex justify-between items-center px-6 py-4">
      <div className="flex items-center">
        <span className="text-xl font-bold">Rust-eze</span>
      </div>

      <div className="text-xl text-gray-800">
        Ruste-eze Audio Streaming
      </div>

      <div className="text-sm text-gray-600">
        Client ID: {clientId}
      </div>
    </div>
  );
};

export default TopBar;
