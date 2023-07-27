import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';
import WebSocketClient from './wscli';
import registerServiceWorker from './registerServiceWorker';

let nextMessageId = 1;
// try {
//   let entry = window.localStorage.getItem('nextMessageId');
//   if (entry) {
//     nextMessageId = parseInt(entry);
//   }
// } catch (e) {
//   console.error(e);
// }

// subscriptions
// let subs = {};

// messages
let msgs = {};

function on_urbit_event(data) {
  if (!msgs.hasOwnProperty(data.id)) {
    console.warn(
      `ws: [on_urbit_event] message received with no corresponding client side queue entry. detail: ${data}`
    );
  }

  console.log(`acking ${data.id} msg-id=${nextMessageId + 1}`);
  // ack the event before doing anything else
  window.ws.send(
    JSON.stringify([
      {
        id: nextMessageId++,
        action: 'ack',
        'event-id': data.id,
      },
    ])
  );

  // remove the message from the queue
  delete msgs[data.id];

  if ('err' in data) {
    console.error(
      `ws: [on_urbit_event] ${data.id} ${data.response} error: ${data.err}`
    );
    // localStorage.set(`error-${msg.id}`, event.data);
    return;
  }

  switch (data.response) {
    case 'diff':
      {
        if (msgs[data.id].handler) {
          msgs[data.id].handler(data.json);
        } else {
          console.warn(`ws: [on_urbit_event] no handler for ${data}`);
        }
      }
      break;

    case 'quit':
      {
        console.log(`ws: [on_urbit_event] quit received`);
      }
      break;

    case 'poke':
      {
        console.log(`ws: [on_urbit_event] poke received`);
      }
      break;

    default:
      console.warn(
        `ws: [on_urbit_event] ${data.id} unrecognized message 'response' field => %o`,
        data
      );
      break;
  }
}

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
    let data = undefined;
    try {
      data = JSON.parse(event.data);
    } catch (e) {
      console.error(e);
      data = event;
    }

    console.log('ws: [onmessage] event received. %o', data);

    // assume that if the message has response and id fields that it is an urbit
    //  ship response. see: https://developers.urbit.org/reference/arvo/eyre/external-api-ref#responses
    if (typeof data === 'object' && 'id' in data && 'response' in data) {
      on_urbit_event(data);
    } else {
      // all other data coming in from socket has been echo'd back to us
      //  this is considered a holon response; therefore for now simply print the value
      console.log('ws: [onmessage] - event is a holon response');
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
    return action;
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
    let payload = message;

    // for now assuming that anything that is an array, is a valid
    //  urbit action message
    if (Array.isArray(message)) {
      // add the message to the queue to properly ack it when
      //   the holon responds
      payload = ws.prepare(message);
    }

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
