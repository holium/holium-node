import React, { useState, useRef } from 'react';
import { useObserver } from 'mobx-react-lite';

const ws = 'ws://';
const nodeUrl = '127.0.0.1:3030';

export default function WebSocketClient() {
  const [url, setUrl] = useState('');
  const [status, setStatus] = useState('disconnected');
  const socketRef = useRef();

  const connect = (wsUrl) => {
    //`${ws}${nodeUrl}/hol/ws`
    try {
      socketRef.current = new WebSocket(wsUrl);

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
      };
    } catch (e) {
      console.error(e);
    }
  };

  const disconnect = () => {
    socketRef.current.send(serialize({ type: 'disconnect' }));
    socketRef.current.close();
  };

  const serialize = (data) => {
    return JSON.stringify(data);
  };

  const unserialize = (data) => {
    try {
      return JSON.parse(data.toString());
    } catch (e) {
      return undefined;
    }
  };

  const responseParser = (response) => {
    console.log(response);
  };

  return useObserver(() => (
    <div className='App'>
      <header>
        <h2>Holium node - websocket test ui</h2>
        <div
          style={{
            display: 'flex',
            flexDirection: 'row',
            gap: 8,
            justifyContent: 'center',
          }}
        >
          <input
            type='text'
            placeholder='Enter websocket url'
            value={url}
            onChange={(evt) => {
              evt.stopPropagation();
              setUrl(evt.target.value);
            }}
          />
        </div>
        <div style={{ marginTop: 8, marginBottom: 8 }}>
          Holium node: {status}
        </div>
        <div
          style={{
            display: 'flex',
            flexDirection: 'row',
            gap: 8,
            justifyContent: 'center',
          }}
        >
          <button
            disabled={status === 'connected'}
            onClick={() => connect(url)}
          >
            Connect
          </button>
          <button
            disabled={status === 'disconnected'}
            onClick={() => disconnect()}
          >
            Disconnect
          </button>
        </div>
      </header>
    </div>
  ));
}
