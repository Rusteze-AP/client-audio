# client-audio
This client has a web page to stream audio files. The audio files can be stored locally or on other nodes. When the user wants to play a remote song, the client asks the server for the node that has the song (which could also be the server itself). After the server's response, the client requests the song chunks from the peer for streaming.

The back-end is entirely written in Rust. One thread handles all messages between nodes and manages the client's state. Another thread is a Rocket endpoint that handles requests from the front-end.

The front-end is a React app using TypeScript. It is a single page that displays all available audio files in the network and plays the sound through a custom bar.

The streaming protocol used is HTTP Live Streaming (HLS). In this protocol, the audio file is divided into multiple segments, and a playlist file serves as the manifest that defines which segment corresponds to the required song timing. During streaming, the client requests a set of segments to buffer the stream, and when the user reaches the end of the buffer, it requests additional segments. If the network is unreliable, the streaming will pause until the segments are loaded, preventing crashes.