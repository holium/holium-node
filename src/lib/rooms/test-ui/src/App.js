import React, { useState, useEffect, useRef } from 'react';
import logo from './logo.svg';
import './App.css';
import Peer from 'simple-peer'

const debug = require('debug')('screen-share:app')

const enableTrickle = true

function App() {
  // get query param ?serverId=xxx&path=xxx
  const urlParams = new URLSearchParams(window.location.search);
  const serverId = urlParams.get('serverId');
  const path = urlParams.get('path');
  const [roomName, setRoomName] = useState('test'); // room-~zod
  const [status, setStatus] = useState('disconnected');
  const [peers, setPeers] = useState({});
  const [present, setPresent] = useState([]);
  const [rooms, setRooms] = useState([]);
  const [currentRoom, setCurrentRoom] = useState(null);
  const [stream, setStream] = useState(null);
  const [mediaErr, setMediaErr] = useState(null);
  const [username, setUsername] = useState(serverId); 
  const [payload, setPayload] = useState(`
    {
      "type": "create-room"
    }
    {
      "type": "enter-room",
      "rid": "room-~zod"
    }
  `);
  const videoRef = useRef();
  const socketRef = useRef();


  useEffect(() => {
    const getMedia = async () => {
      if (!videoRef.current) return;
      try {
        const mediaStream = await navigator.mediaDevices.getUserMedia({ video: true });
        // const mediaStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        if(videoRef.current) {
            videoRef.current.srcObject = mediaStream;
        }
      } catch (err) {
        console.error('Could not get user media:', err);
      }
    }

    if (username) {
      connect();
    }
    
    fetch('http://localhost:3030/hol/rooms').then(res => res.json()).then(res => {
      setRooms(res);
    }).catch(err => {
      console.error('Could not get rooms:', err);
    });

    getMedia();
  }, []);

  useEffect(() => {
    if (stream && videoRef.current && !videoRef.current.srcObject) {
      debug('set video stream', videoRef.current, stream)
      videoRef.current.srcObject = stream
    }
    if (peers.length > 0) {
      attachPeerVideos();
    }
  }, [stream, peers]);

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
    // if (!currentRoom) {
    //   debug('no current room')
    //   return
    // }

    const peer = new Peer({initiator: initiator, trickle: enableTrickle, stream})

    peer.on('signal', (signal) => {
      const msg = { type: "signal", rid: currentRoom.rid, signal, to: peerId, from: username }
      debug('peer signal sent', msg)
      socketRef.current.send(serialize(msg))
    })
  
    peer.on('stream', (stream) => {
      debug('Got peer stream!!!', peerId, stream)
      peer.stream = stream
      setPeerState(peerId, peer)
    })

    peer.on('connect', () => {
      debug('Connected to peer', peerId)
      peer.connected = true
      setPeerState(peerId, peer)
      peer.send(serialize({
        msg: 'hey man!'
      }))
    })

    peer.on('data', data => {
      debug('Data from peer', peerId, unserialize(data))
    })

    peer.on('error', (e) => {
      debug('Peer error %s:', peerId, e);
    })

    setPeerState(peerId, peer)

    return peer
  }

  const destroyPeer = (peerId) => {
    const peers = {...peers}
    delete peers[peerId]
    setPeers({
      ...peers
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
    const peers = {...peers}
    peers[peerId] = peer
    setPeers({
     ...peers
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
      setStatus('connected');
      socketRef.current.send(serialize({ type: 'connect' }));
    };

    socketRef.current.onmessage = function incoming(message) {
      const parsedMessage = unserialize(message.data);
      responseParser(parsedMessage);
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
      disconnect();
    }
  };

  const disconnect = () => {
    socketRef.current.send(serialize({ type: 'disconnect' }));
    socketRef.current.close();
  };

  const isInRoom = (rid) => {
    if (!currentRoom) {
      return false;
    }
    return currentRoom.rid === rid;
  };

  const isCreator = (rid) => {
    const room = rooms.find((room) => room.rid === rid);
    if (!room) {
      return false;
    }
    return room.creator === username;
  };

  const createRoom = (roomName) => {
    socketRef.current.send(serialize({ type: 'create-room', rid: `${username}/${roomName}`, title: roomName }));
  };

  const deleteRoom = (rid) => {
    socketRef.current.send(serialize({ type: 'delete-room', rid }));
  };

  const responseParser = (response) => {
    switch (response.type) {
      case 'rooms': 
        let currentRoom = null;
        response.rooms.forEach((room) => {
          if (room.creator === username) {
            currentRoom = room;
            setPresent(room.present);
          }
          if (room.present.includes(username)) {
            currentRoom = room;
            setPresent(room.present);
          }
        });
        setRooms(response.rooms);
        setCurrentRoom(currentRoom);
        break;
      case 'room-entered':
        setCurrentRoom(response.room);
        setPresent(response.room.present);
        break;
      case 'room-left':
        setPresent(response.room.present);
        break;
      case 'room-deleted':
        const removeRid = response.rid;
        setRooms(rooms.filter((room) => room.rid !== removeRid));
        break;
      case 'signal':
        const { signal, from } = response;
        console.log('got signal', signal, from);
        // TODO handle signal
        console.log(peers)
        // peers[from].signal(signal);
        break;
    }
  };


  const sendPayload = () => {
    try {
      let parsedPayload = unserialize(payload);
      socketRef.current.send(serialize(parsedPayload));
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

        </div>
        <div style={{marginTop: 8, marginBottom: 8}}>Holium node: {status}</div>
        <div style={{ display: 'flex', flexDirection: 'row', gap: 8, justifyContent: 'center' }}>
          <button disabled={status === 'connected'} onClick={() => connect()}>Connect</button>
          <button  disabled={status === 'disconnected'} onClick={() => disconnect()}>Disconnect</button>
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
      
      <div style={{ display: 'flex', flexDirection: 'row', width: 900, marginTop: 30 }}>
        <div className='rooms-list'>
          <div style={{ display: 'flex', flexDirection: 'row', gap: 8, padding: '8px 8px' }}>
            <input
              type="text"
              placeholder="Enter room name"
              value={roomName}
              onChange={(evt) => {
                evt.stopPropagation()
                setRoomName(evt.target.value);
              }}
            />
          <button onClick={() => createRoom(roomName)}>Create</button>
          {/* <button onClick={() => deleteRoom()}>Delete room</button> */}
        </div>
          <div style={{ display: 'flex', flexDirection: 'column'}}>
            {rooms.map((room) => {
              return (
                <div key={room} className="room-row">
                  <span>{room.rid}</span>
                  {!isInRoom(room.rid) ? (
                    <button onClick={() => {
                      socketRef.current.send(serialize({ type: 'enter-room', rid: room.rid }));
                    }}>Enter</button>
                  ) : (
                      isCreator(room.rid) ? (
                        <button onClick={() => deleteRoom(room.rid)}>Delete</button>
                      ) : (
                      <button onClick={() => {
                        socketRef.current.send(serialize({ type: 'leave-room', rid: room.rid }));
                      }}>Leave</button>
                      )
                  )}
                </div>
              )
            })}
          </div>
        </div>
        <div id="our-video">
          <video style={{width: 400}} ref={videoRef} autoPlay playsInline />
        </div>
        <div className="peers">{present.map((peer) => {
          let callbutton = null;
          if (peer !== username) {
            callbutton = <button onClick={() => createPeer(peer, true, stream)}>Call</button>
          }
          return (
            <div className="peer-row" key={peer}>
              {peer}
              {callbutton}
            </div>
          )
        })}</div>
      </div>
    </div>
  );
}

export default App;