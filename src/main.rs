use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::data_channel::RTCDataChannel;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // Everything below is the WebRTC-rs API! Thanks for using it ❤️.

    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();

    // Register default codecs
    m.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    // Create a datachannel with label 'data'
    let data_channel = peer_connection.create_data_channel("data", None).await?;

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
            // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
            // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
            println!("Peer Connection has gone to failed exiting");
            let _ = done_tx.try_send(());
        }

        Box::pin(async {})
    }));

    /*
    // Register channel opening handling
    let d1 = Arc::clone(&data_channel);
    data_channel.on_open(Box::new(move || {
        println!("Data channel '{}'-'{}' open.    Random messages will now be sent to any connected DataChannels every 5 seconds", d1.label(), d1.id());

        let d2 = Arc::clone(&d1);
        Box::pin(async move {
            d2.send_text(String::from("send text fun")).await;
        })
    }));
    */

     // Register data channel creation handling
    peer_connection
        .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {d_label} {d_id}");

            // Register channel opening handling
            Box::pin(async move {
                let d2 = Arc::clone(&d);
                let d_label2 = d_label.clone();
                let d_id2 = d_id;
                d.on_open(Box::new(move || {
                    println!("Data channel '{d_label2}'-'{d_id2}' open. Random messages will now be sent to any connected DataChannels every 5 seconds");

                    Box::pin(async move {
                        let mut result = Result::<usize,webrtc::Error>::Ok(0);
                        while result.is_ok() {
                            let timeout = tokio::time::sleep(Duration::from_secs(5));
                            tokio::pin!(timeout);

                            tokio::select! {
                                _ = timeout.as_mut() =>{
                                    println!("Sending message");
                                    result = d2.send_text(String::from("1808")).await.map_err(Into::into);
                                }
                            };
                        }
                    })
                }));

                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{d_label}': '{msg_str}'");
                    Box::pin(async {})
                }));
            })
        }));


    /*
    // Register text message handling
    let d_label = data_channel.label().to_owned();
    data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
        println!("Message from DataChannel '{d_label}': '{msg_str}'");
        Box::pin(async {})
    }));
    */

    // Create an offer to send to the browser
    let offer = peer_connection.create_offer(None).await?;
    /*let offer = serde_json::from_str::<RTCSessionDescription>(r#"
    {"type":"offer","sdp":"v=0\r\no=- 1405354167094100842 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application 59648 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 179.108.161.231\r\na=candidate:399114311 1 udp 2113937151 5e0e0010-852b-4163-a212-691b1d19f2e1.local 59648 typ host generation 0 network-cost 999\r\na=candidate:485314046 1 udp 2113939711 b55180b3-54f2-44cb-9c2a-168d6670b47d.local 51863 typ host generation 0 network-cost 999\r\na=candidate:904817005 1 udp 1677729535 179.108.161.231 59648 typ srflx raddr 0.0.0.0 rport 0 generation 0 network-cost 999\r\na=candidate:248516973 1 udp 1677732095 2804:ef4:5239:8a00:5452:8451:f0a6:7 51863 typ srflx raddr :: rport 0 generation 0 network-cost 999\r\na=ice-ufrag:B29Q\r\na=ice-pwd:/cx2P/2T8MYsHpGpSebrrh0Y\r\na=ice-options:trickle\r\na=fingerprint:sha-256 9A:0D:98:A5:64:B3:89:0F:E6:38:1A:0B:06:1E:E7:7B:EF:78:14:59:B2:DE:C0:79:F3:9A:E1:2F:92:B2:A8:5C\r\na=setup:actpass\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"}
    "#)?;*/

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(offer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        println!("local_description {:?}", json_str);
    } else {
        println!("generate local_description failed!");
    }


    //let answer = serde_json::from_str::<RTCSessionDescription>("")?;

    // Apply the answer as the remote description
    //peer_connection.set_remote_description(answer).await?;

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("ctrl_c");
        }
    }


    peer_connection.close().await?;

    Ok(())
}
