const WebSocket = require('ws');


// interface ClientOptions extends SecureContextOptions {
//     protocol?: string | undefined;
//     followRedirects?: boolean | undefined;
//     generateMask?(mask: Buffer): void;
//     handshakeTimeout?: number | undefined;
//     maxRedirects?: number | undefined;
//     perMessageDeflate?: boolean | PerMessageDeflateOptions | undefined;
//     localAddress?: string | undefined;
//     protocolVersion?: number | undefined;
//     headers?: { [key: string]: string } | undefined;
//     origin?: string | undefined;
//     agent?: Agent | undefined;
//     host?: string | undefined;
//     family?: number | undefined;
//     checkServerIdentity?(servername: string, cert: CertMeta): boolean;
//     rejectUnauthorized?: boolean | undefined;
//     maxPayload?: number | undefined;
//     skipUTF8Validation?: boolean | undefined;
// }

const connect = () => {
    const ws = new WebSocket('ws://localhost:3030/signaling?serverId=~lomder-librun');
    ws.removeAllListeners();
    ws.on('open', function open() {
      console.log('connected');
      ws.send(JSON.stringify({ type: 'ping' }));
    });

    ws.on('message', function incoming(data) {
      const message = JSON.parse(data.toString());
      console.log(message)
    });

    ws.on('close', function close() {
        console.log('disconnected');
        setTimeout(connect, 5000); // Try to reconnect after 5 seconds
    });

    ws.on('error', function error(err) {
        console.error('Error occurred:', err);
        ws.close();
    });
    // on sigkill close the connection
    process.on('SIGINT', function () {
      ws.close();
      // kill the process
      process.exit();
    });
};


connect();