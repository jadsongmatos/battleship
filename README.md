# battleship

This is a Rust program that demonstrates the usage of the WebRTC-rs library to create a PeerConnection, establish an ICE connection with a remote peer, and exchange data through DataChannels. The program does the following:

1.  Creates a MediaEngine object and registers default codecs.
2.  Creates an InterceptorRegistry and registers default interceptors.
3.  Builds an API object with the MediaEngine and InterceptorRegistry.
4.  Prepares the RTCConfiguration, including the ICE server to use.
5.  Creates a new RTCPeerConnection using the API object.
6.  Creates a DataChannel with the label "data".
7.  Sets up a handler for the PeerConnection state change, closing the connection when the state becomes "failed".
8.  Sets up a handler for DataChannel creation, opening, and message handling.
9.  Creates an offer, sets the local description, and waits for ICE gathering to complete.
10.  Prints the local description as a JSON string, which can be shared with the remote peer.
11.  Waits for the user to press Ctrl-C to stop the program or for the connection to fail.
12.  Closes the RTCPeerConnection.

This program is essentially a basic WebRTC server that can be used for testing purposes. It listens for connections, creates DataChannels, and exchanges messages with the remote peer over the DataChannels. Note that this code does not include the remote peer setup or signaling to exchange the offer and answer. You would need a separate mechanism for exchanging the offer and answer between the remote peer and the server.

https://github.com/webrtc-rs/webrtc/tree/master/examples/examples/data-channels