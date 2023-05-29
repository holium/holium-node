import React, { useState, useEffect, useRef } from 'react';
import { patp2dec } from '@urbit/aura';
import { toJS } from 'mobx';
import logo from './logo.svg';
import './App.css';
import Peer from 'simple-peer'
import { useObserver } from 'mobx-react-lite';
import PeerStore from './PeerStore';

const enableTrickle = true


function App() {
  // get query param ?serverId=xxx&path=xxx
  const urlParams = new URLSearchParams(window.location.search);
  const serverId = urlParams.get('serverId');
  const path = urlParams.get('path');
  const [roomName, setRoomName] = useState('test'); // room-~zod
  const [status, setStatus] = useState('disconnected');
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
  const currentRoomRef = useRef();
  const peersRef = useRef(PeerStore.peers);
  const roomsRef = useRef(rooms);
  const streamRef = useRef();


  useEffect(() => {
    const getMedia = async () => {
      if (!videoRef.current) return;
      try {
        const mediaStream = await navigator.mediaDevices.getUserMedia({ video: true });
        // const mediaStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        if(videoRef.current) {
          videoRef.current.srcObject = mediaStream;
          setStream(mediaStream);
          streamRef.current = mediaStream;
        }
      } catch (err) {
        console.error('Could not get user media:', err);
      }
    }

    if (username) {
      connect();
    }
    
    fetch('http://localhost:3030/hol/rooms').then(res => res.json()).then(res => {
      setRoomsState(res);
    }).catch(err => {
      console.error('Could not get rooms:', err);
    });

    getMedia();
  }, []);

  useEffect(() => {
    if (stream && videoRef.current && !videoRef.current.srcObject) {
      console.log('set video stream', videoRef.current, stream)
      videoRef.current.srcObject = stream
    }
    if (peersRef.current.size > 0) {
      attachPeerVideos();
    }
  }, [stream]);

  const attachPeerVideos = () => {
    Object.entries(peersRef.current).forEach(entry => {
      const [peerId, peer] = entry
      if (peer.video && !peer.video.srcObject && peer.stream) {
        console.log('setting peer video stream', peerId, peer.stream)
        peer.video.setAttribute('data-peer-id', peerId)
        peer.video.srcObject = peer.stream
      }
    })
    // setPeers(newPeers);
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
    return Object.entries(peersRef.current).map(entry => {
      const [peerId, peer] = entry
      console.log('render peer', peerId, peer, entry)
      return <div key={peerId}>
        <video ref={video => peer.video = video}></video>
      </div>
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
    PeerStore.setPeer(peerId, peer);
  }

  const setRoomsState = (rooms) => {
    roomsRef.current = rooms;
    setRooms(rooms);
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

  const isInitiatorCheck = (from, to) => {
    return patp2dec(from) > patp2dec(to);
  };

  const createRoom = (roomName) => {
    socketRef.current.send(serialize({ type: 'create-room', rid: `${username}/${roomName}`, title: roomName }));
  };

  const deleteRoom = (rid) => {
    socketRef.current.send(serialize({ type: 'delete-room', rid }));
  };

  const leaveRoom = (rid) => {
    socketRef.current.send(serialize({ type: 'leave-room', rid }));
    // remove self from room present list
    const updatedRoom = rooms.find((room) => room.rid === rid);
    if (updatedRoom) {
      updatedRoom.present = updatedRoom.present.filter((user) => user !== username);
    }
    setCurrentRoom(undefined);
    setRoomsState(rooms.map((room) => room.rid === rid ? updatedRoom : room));
  };


  const createPeer = (peerId, initiator, stream) => {
    console.log('createPeer', peerId, initiator, stream)

    const peer = new Peer({
      initiator: initiator, trickle: enableTrickle, stream, config: {
        iceServers: [
          {
            username: 'realm',
            credential: 'zQzjNHC34Y8RqdLW',
            urls: ['turn:coturn.holium.live:3478'],
          },
          {
            urls: ['stun:coturn.holium.live:3478'],
          },
        ],
    
      }
    });

    peer.on('signal', (signal) => {
      const msg = { type: "signal", rid: currentRoomRef.current.rid, signal, to: peerId, from: username }
      socketRef.current.send(serialize(msg))
    })
  
    peer.on('stream', (stream) => {
      console.log('Got peer stream!!!', peerId, stream)
      peer.addStream(stream)
      document.getElementById(`peer-video-${peerId}`).srcObject = stream
      setPeerState(peerId, peer)
    })

    peer.on('connect', () => {
      console.log('Connected to peer', peerId)
      setPeerState(peerId, peer)
      peer.send(serialize({
        type: 'peer-message',
        msg: 'hey man!'
      }))
    })

    peer.on('data', data => {
      console.log('Data from peer', peerId, unserialize(data))
    })

    peer.on('error', (e) => {
      console.log('Peer error %s:', peerId, e);
    })

    setPeerState(peerId, peer)

    return peer
  }

  const destroyPeer = (peerId) => {
    PeerStore.deletePeer(peerId);
  }

   const onPeerSignal = (from, data) => {
     try {
      const peer = peersRef.current[from];
      peer.signal(data)
    } catch(e) {
      console.error('sigal error', e)
    }
  }


  const responseParser = (response) => {
    switch (response.type) {
      case 'rooms': 
        setRoomsState(response.rooms);
        break;
      case 'room-created':
        console.log('room created', response.room);
        currentRoomRef.current = response.room;
        setCurrentRoom(response.room);
        break;
      case 'room-entered':
        console.log('room entered', response.room);
        currentRoomRef.current = response.room;
        setCurrentRoom(response.room);
        // if we entered a room, we need to create a peer for each user in the room
        const peers = response.room.present.filter((user) => user !== username);
        peers.forEach((peerId) => {
          const initiator = isInitiatorCheck(username, peerId);
          const peer = createPeer(peerId, initiator, streamRef.current);
          peersRef.current[peerId] = peer;
        });
        
        break;
      case 'room-left':
        // replace room in rooms array with response.room
        const room = response.room;
        const currentRoomLeft = currentRoomRef.current;
        if (currentRoomLeft.rid === room.rid) {
          if (response.peer_id === username) {
            setCurrentRoom(undefined);
            currentRoomLeft.current = undefined;
          } else {
            // someone left the room
            console.log('someone left the room', response);
            destroyPeer(response.peer_id);
            const updatedRoom = roomsRef.current.find((room) => room.rid === response.rid);
            if (updatedRoom) {
              updatedRoom.present = updatedRoom.present.filter((user) => user !== username);
            }
            setRoomsState(roomsRef.current.map((room) => room.rid === response.rid ? updatedRoom : room));
            currentRoomLeft.current = updatedRoom;
            setCurrentRoom(updatedRoom);
          }
        }
        break;
      case 'room-deleted':
        const removeRid = response.rid;
        setRoomsState(rooms.filter((room) => room.rid !== removeRid));
        const currentRoomDeleted = currentRoomRef.current;
        if (currentRoomDeleted.rid === removeRid) {
          currentRoomDeleted.present.forEach((peerId) => {
            if(peerId !== username) destroyPeer(peerId);
          });
          setCurrentRoom(undefined);
          currentRoomDeleted.current = undefined;

        }
        break;
      case 'signal':
        const { signal, from } = response;
        const isInitiator = isInitiatorCheck(username, from);

        if (currentRoomRef.current && currentRoomRef.current.present.includes(from)) {
          const peer = peersRef.current[from] || createPeer(from, isInitiator, streamRef.current);
          setPeerState(from, peer)
          onPeerSignal(from, signal)
        }
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

  return useObserver(() => (
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
                        <button onClick={() => leaveRoom(room.rid)}>Leave</button>
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
        <div className="peers">{currentRoom && currentRoom.present.map((peer) => {
          let callbutton = null;
          let video = null;
          if (peer !== username) {
            {/* callbutton = <button disabled={PeerStore.peers[peer] && PeerStore.peers[peer].connected} onClick={() => createPeer(peer, isInitiatorCheck(username, peer), stream)}>Call</button>             */}
            video = <video id={`peer-video-${peer}`} style={{width: 200}} autoPlay playsInline />
          }
          return (
            <div className="peer-row" key={peer}>
              <div style={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between' }}>
                {peer}
                {/* {callbutton} */}
              </div>
              {video}
            </div>
          )
        })}</div>
      </div>
    </div>
  ));
}

export default App;