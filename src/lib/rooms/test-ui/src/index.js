import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';
import WebSocketClient from './wscli';
import registerServiceWorker from './registerServiceWorker';

// ReactDOM.render(<App />, document.getElementById('root'));
ReactDOM.render(<WebSocketClient />, document.getElementById('root'));
registerServiceWorker();
