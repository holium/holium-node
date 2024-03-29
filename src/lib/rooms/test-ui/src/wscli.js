import React, { useEffect, useState, useRef } from 'react';
import { useObserver } from 'mobx-react-lite';

const ws = 'ws://';
const nodeUrl = '127.0.0.1:3030';

let creatorWs = null;

const randomNumberInRange = (min, max) => {
  return Math.floor(Math.random() * (max - min + 1) + min);
};

const stopTests = () => {
  creatorWs.send(
    JSON.stringify({
      type: 'leave-room',
      rid: '~lodlev-migdev/test-room',
    })
  );
};
const runTests = () => {
  const serverIds = [
    '~lodlev-migdev',
    '~ralbes-mislec-lodlev-migdev',
    '~hobpec-sopped-lodlev-migdev',
    '~locwyd-mocrut-lodlev-migdev',
  ];

  const creatorUrl = `wss://node-test.holium.live/signaling?serverId=${serverIds[0]}`;
  console.log(`connecting to websocket ${creatorUrl}...`);
  creatorWs = new WebSocket(creatorUrl);
  creatorWs.onopen = (ev) => {
    console.log('websocket opened');
    creatorWs.send(JSON.stringify({ type: 'connect' }));
  };
  creatorWs.onerror = (ev) => {
    console.error('websocket error => %o', ev);
  };
  creatorWs.onclose = (ev) => {
    console.warn('websocket closed');
  };
  creatorWs.onmessage = (ev) => {
    console.log('websocket message');
    console.log(ev.data);
    const data = JSON.parse(ev.data);
    switch (data.type) {
      case 'connected':
        creatorWs.send(
          JSON.stringify({
            type: 'create-room',
            rtype: 'media',
            rid: '~lodlev-migdev/test-room',
            title: 'Test Room',
            path: '~lodlev-migdev/test-room',
          })
        );
        break;

      case 'room-created':
        for (let i = 0; i < 20; i++) {
          const r1 = randomNumberInRange(0, 3);
          const url = `wss://node-test.holium.live/signaling?serverId=${serverIds[r1]}`;
          console.log(`connecting to websocket ${url}...`);
          const ws = new WebSocket(url);
          ws.onopen = (ev) => {
            console.log('websocket opened');
            ws.send(JSON.stringify({ type: 'connect' }));
          };
          ws.onerror = (ev) => {
            console.error('websocket error => %o', ev);
          };
          ws.onclose = (ev) => {
            console.warn('websocket closed');
          };
          ws.onmessage = (ev) => {
            console.log('websocket message');
            console.log(ev.data);
            const data = JSON.parse(ev.data);
            switch (data.type) {
              case 'connected': {
                ws.send(
                  JSON.stringify({
                    type: 'enter-room',
                    rid: '~lodlev-migdev/test-room',
                  })
                );
                break;
              }
            }
          };
        }
        break;
    }
  };
};

export default function WebSocketClient() {
  const [url, setUrl] = useState('');
  const [status, setStatus] = useState('disconnected');
  const [payload, setPayload] = useState('');
  const socketRef = useRef();

  const sendPayload = () => {
    try {
      let parsedPayload = unserialize(payload);
      if (parsedPayload !== undefined) {
        socketRef.current.send_raw(parsedPayload);
      } else {
        console.error('error parsing payload');
      }
      // } else {
      //   socketRef.current.send_raw(payload);
      // }
    } catch (e) {
      console.log('invalid json', e);
    }
  };

  const connect = (wsUrl) => {
    //`${ws}${nodeUrl}/hol/ws`
    try {
      document.cookie =
        'urbauth-~ralbes-mislec-lodlev-migdev=0v7.pmchc.of0sj.lhqur.nbrig.pkf9q; Path=/; Max-Age=604800';

      socketRef.current = window.connectWs(wsUrl);
      // socketRef.current = new WebSocket(wsUrl);

      // socketRef.current.onopen = function open() {
      //   setStatus('connected');
      //   socketRef.current.send(serialize({ type: 'connect' }));
      // };

      // socketRef.current.onmessage = function incoming(message) {
      //   const parsedMessage = unserialize(message.data);
      //   responseParser(parsedMessage);
      // };

      // socketRef.current.onclose = function close() {
      //   console.log('disconnected');
      //   setStatus('disconnected');
      //   // setTimeout(connect, 5000); // Try to reconnect after 5 seconds
      // };

      // socketRef.current.onerror = function error(err) {
      //   console.error('Error occurred:', err);
      //   setStatus('error');
      //   socketRef.current.close();
      // };
      // // on sigkill close the connection
      // window.onbeforeunload = function () {
      //   disconnect();
      // };
    } catch (e) {
      console.error(e);
    }
  };

  const disconnect = () => {
    // socketRef.current.send(serialize({ type: 'disconnect' }));
    socketRef.current.close();
  };

  const serialize = (data) => {
    return JSON.stringify(data);
  };

  const unserialize = (data) => {
    try {
      return JSON.parse(data); //.toString());
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
            // disabled={status === 'disconnected'}
            onClick={() => disconnect()}
          >
            Disconnect
          </button>
        </div>
      </header>
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 8,
          alignItems: 'center',
        }}
      >
        <textarea
          style={{ marginTop: 16 }}
          type='text'
          cols={50}
          rows={10}
          placeholder='Enter message payload'
          value={payload}
          onChange={(evt) => {
            evt.stopPropagation();
            // text area value
            setPayload(evt.target.value);
          }}
        />
        <button onClick={() => sendPayload()}>Send payload</button>
        <button onClick={() => runTests()}>Run Tests</button>
        <button onClick={() => stopTests()}>Stop Tests</button>
      </div>
    </div>
  ));
}
