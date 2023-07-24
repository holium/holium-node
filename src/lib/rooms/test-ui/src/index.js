import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';
import WebSocketClient from './wscli';
import registerServiceWorker from './registerServiceWorker';

let nextMessageId = 1;
try {
  let entry = window.localStorage.getItem('nextMessageId');
  if (entry) {
    nextMessageId = parseInt(entry);
  }
} catch (e) {
  console.error(e);
}

// subscriptions
// let subs = {};

// messages
let msgs = {};

window.connectWs = (url) => {
  if (window.ws) {
    console.warn(
      `ws: [connect] ws connection already exists. closing and restarting connection...`
    );
    window.ws.close();
  }
  let ws = new WebSocket(url);

  ws.onopen = (event) => {
    console.log('ws: [onopen] web socket opened %o', event);
  };

  ws.onmessage = (event) => {
    console.log('ws: [onmessage] message received. %o', event);

    let msg = JSON.parse(event.data);

    if (!msgs.hasOwnProperty(msg.id)) {
      console.warn(
        `ws: [onmessage] message received with no corresponding client side queue entry. detail: ${msg}`
      );
    }

    console.log(`acking ${msg.id} msg-id=${nextMessageId + 1}`);
    // ack the event before doing anything else
    ws.send(
      JSON.stringify([
        {
          id: nextMessageId++,
          action: 'ack',
          'event-id': msg.id,
        },
      ])
    );

    // remove the message from the queue
    delete msgs[msg.id];

    if ('err' in msg) {
      console.error(
        `ws: [onmessage] ${msg.id} ${msg.response} error: ${msg.err}`
      );
      window.localStorage.set(`error-${msg.id}`, event.data);
      return;
    }

    switch (msg.response) {
      case 'diff':
        {
          if (msgs[msg.id].handler) {
            msgs[msg.id].handler(msg.json);
          } else {
            console.warn(`ws: [onmessage] no handler for ${msg}`);
          }
        }
        break;

      case 'quit':
        {
          console.log(`ws: [onmessage] quit received`);
        }
        break;

      case 'poke':
        {
          console.log(`ws: [onmessage] poke received`);
        }
        break;

      default:
        console.warn(
          `ws: [onmessage] ${msg.id} unrecognized message 'response' field => %o`,
          msg
        );
        break;
    }
  };

  ws.onclose = (event) => {
    console.log('ws: [onclose] web socket closed %o', event);
    msgs = {};
    // subs = {};
    window.ws = null;
  };

  ws.onerror = (event) => {
    console.log('ws: [onerror] web socket error %o', event);
  };

  ws.poke = (ship, app, mark, json) => {
    let messageId = nextMessageId++;
    let message = {
      id: messageId,
      action: 'poke',
      ship: ship,
      app: app,
      mark: mark,
      json: json,
    };
    let payload = ws.enqueue(message);
    ws.send(JSON.stringify(payload));
  };

  ws.enqueue = (action, handler) => {
    if (msgs.hasOwnProperty(action.id)) {
      let detail = `id=${action.id} exists in message queue`;
      console.error(`ws: [poke] error. ${detail}`);
      return;
    }
    msgs[action.id] = {};
    msgs[action.id].out = action;

    switch (action.action) {
      case 'poke':
      case 'unsubscribe':
      case 'subscribe':
        break;
    }
  };

  ws.prepare = (message) => {
    let result = undefined;
    if (Array.isArray(message)) {
      result = message.map((action) => ({
        ...action,
        id: action.id ? action.id : nextMessageId++,
      }));
      result.forEach((action) => ws.enqueue(action));
    } else {
      if (!message.id) {
        message.id = nextMessageId++;
      }
      result = ws.enqueue(message);
    }
    return result;
  };

  // take a "raw" actions buffer (straight-up actions payload as JSON string)
  //   and ensure it complies with underlying protocol before sending to server
  // note: if the actions array payloads contain an id value, this value will not be overwritten
  //  and will be used as-is. if there is no id, one will be generated based on nextMessageId value
  ws.send_raw = (message) => {
    let payload = ws.prepare(message);
    ws.send(JSON.stringify(payload));
  };

  ws.subscribe = (ship, app, path, handler) => {
    if (!handler) {
      console.error(`ws: [subscribe] error. missing handler.`);
      return;
    }
    let messageId = nextMessageId++;
    let message = {
      id: messageId,
      action: 'subscribe',
      ship: ship,
      app: app,
      path: path,
    };
    let payload = ws.enqueue(message, handler);
    ws.send(JSON.stringify(payload));
  };

  ws.unsubscribe = (subId) => {
    // if (!subs.hasOwnProperty(subId)) {
    //   console.error(
    //     `ws: [unsubscribe] id=${subId} error. requested unsubscribe for non existant subscription`
    //   );
    //   return;
    // }
    let messageId = nextMessageId++;
    let message = {
      id: messageId,
      action: 'unsubscribe',
      subscription: subId,
    };
    let payload = ws.enqueue(message);
    ws.send(JSON.stringify(payload));
  };

  window.ws = ws;

  return ws;
};

window.onbeforeunload = () => {
  console.log('bye web socket');
  window.ws.close();
};

// ReactDOM.render(<App />, document.getElementById('root'));
ReactDOM.render(<WebSocketClient />, document.getElementById('root'));
registerServiceWorker();
