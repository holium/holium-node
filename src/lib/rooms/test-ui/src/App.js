import React, { useState, useEffect, useRef } from 'react';
import logo from './logo.svg';
import './App.css';
import Peer from 'simple-peer'

const debug = require('debug')('screen-share:app')

const enableTrickle = true

function App() {
  const [status, setStatus] = useState('disconnected');
  const [peers, setPeers] = useState({});
  const [stream, setStream] = useState(null);
  const [mediaErr, setMediaErr] = useState(null);
  const [username, setUsername] = useState(""); 
  const [payload, setPayload] = useState("");
  const videoRef = useRef();
  const socketRef = useRef();

  // useEffect(() => {
  //   getMedia((stream) => {
  //     setStream(stream);
  //   }, err => {
  //     setMediaErr('Could not access webcam');
  //     debug('getMedia error', err);
  //   });
  // }, []);

  // useEffect(() => {
  //   if (stream && videoRef.current && !videoRef.current.srcObject) {
  //     debug('set video stream', videoRef.current, stream)
  //     videoRef.current.srcObject = stream
  //   }
  //   attachPeerVideos();
  // }, [stream, peers]);

  const attachPeerVideos = () => {
    let newPeers = {...peers};
    Object.entries(newPeers).forEach(entry => {
      const [peerId, peer] = entry
      if (peer.video && !peer.video.srcObject && peer.stream) {
        debug('setting peer video stream', peerId, peer.stream)
        peer.video.setAttribute('data-peer-id', peerId)
        peer.video.srcObject = peer.stream
      }
    })
    setPeers(newPeers);
  }

  const handleUsernameChange = (event) => {
    setUsername(event.target.value);
  };

  const getMedia = (callback, err) => {
    const options = { video: true, audio: true }
    if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
      return navigator.mediaDevices.getUserMedia(options)
        .then(stream => callback(stream))
        .catch(e => err(e))
    }
    return navigator.getUserMedia(options, callback,  err)
  }

  const renderPeers = () => {
    return Object.entries(peers).map(entry => {
      const [peerId, peer] = entry
      debug('render peer', peerId, peer, entry)
      return <div key={peerId}>
        <video ref={video => peer.video = video}></video>
      </div>
    })
  }

  const createPeer = (peerId, initiator, stream) => {
    debug('creating new peer', peerId, initiator)

    const peer = new Peer({initiator: initiator, trickle: enableTrickle, stream})

    peer.on('signal', (signal) => {
      const msgId = (new Date().getTime())
      const msg = { msgId, signal, to: peerId }
      debug('peer signal sent', msg)
      this.socket.emit('signal', msg)
    })
  
    peer.on('stream', (stream) => {
      debug('Got peer stream!!!', peerId, stream)
      peer.stream = stream
      this.setPeerState(peerId, peer)
    })

    peer.on('connect', () => {
      debug('Connected to peer', peerId)
      peer.connected = true
      this.setPeerState(peerId, peer)
      peer.send(this.serialize({
        msg: 'hey man!'
      }))
    })

    peer.on('data', data => {
      debug('Data from peer', peerId, this.unserialize(data))
    })

    peer.on('error', (e) => {
      debug('Peer error %s:', peerId, e);
    })

    this.setPeerState(peerId, peer)

    return peer
  }

  const destroyPeer = (peerId) => {
    const peers = {...this.state.peers}
    delete peers[peerId]
    this.setState({
      peers
    })
  }

  const serialize = (data) => {
    return JSON.stringify(data)
  }

  const unserialize = (data) => {
    try {
      return JSON.parse(data.toString())
    } catch(e) {
      return undefined
    }
  }

  const setPeerState = (peerId, peer) => {
    const peers = {...this.state.peers}
    peers[peerId] = peer
    this.setState({
      peers
    })
  }

  const signalPeer = (peer, data) => {
    try {
      peer.signal(data)
    } catch(e) {
      debug('sigal error', e)
    }
  }

  const connect = () => {
    socketRef.current = new WebSocket(`ws://localhost:3030/signaling?serverId=${username}`);
    socketRef.current.onopen = function open() {
      console.log('connected');
      setStatus('connected');
      socketRef.current.send(JSON.stringify({ type: 'ping' }));
    };

    socketRef.current.onmessage = function incoming(message) {
      console.log(message)
      const parsedMessage = JSON.parse(message.data);
      console.log(parsedMessage)
    };

    socketRef.current.onclose = function close() {
      console.log('disconnected');
      setStatus('disconnected');
        // setTimeout(connect, 5000); // Try to reconnect after 5 seconds
    };

    socketRef.current.onerror = function error(err) {
      console.error('Error occurred:', err);
      setStatus('error');
      socketRef.current.close();
    };
    // on sigkill close the connection
    window.onbeforeunload = function () {
      socketRef.current.close();
    }
  };

  const disconnect = () => {
    socketRef.current.send(JSON.stringify({ type: 'disconnect' }));
    socketRef.current.close();
  };

  const sendPayload = () => {
    try {
      let parsedPayload = JSON.parse(payload);
      socketRef.current.send(JSON.stringify(parsedPayload));
    } catch (e) {
      console.log('invalid json', e)
    }
  };

  return (
    <div className="App">
      <header>
        <h2>Holium node - rooms test ui</h2>
        <div style={{display: 'flex', flexDirection: 'row', gap: 8, justifyContent: 'center'}}>
        <input
          type="text"
          placeholder="Enter patp"
          value={username}
          onChange={(evt) => {
            evt.stopPropagation()
            setUsername(evt.target.value);
          }}
        />
        <button onClick={() => connect()}>Connect</button>
        <button onClick={() => disconnect()}>Disconnect</button>

        </div>
      </header>
      <div style={{display: 'flex', flexDirection: 'column', gap: 8, alignItems: 'center'}}>
        <textarea
          style={{marginTop: 16}}
          type="text"
          cols={50}
          rows={10}
          placeholder="Enter message payload"
          value={payload}
          onChange={(evt) => {
            evt.stopPropagation();
            // text area value
            setPayload(evt.target.value);
            
          }}
        />
        <button onClick={() => sendPayload()}>Send payload</button>
        </div>
      {mediaErr && (
        <p className="error">{mediaErr}</p>
      )}
      <div id="peers">{renderPeers()}</div>
    </div>
  );
}

export default App;