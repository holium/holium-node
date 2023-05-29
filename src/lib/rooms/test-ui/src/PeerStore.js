import { makeAutoObservable } from 'mobx';

class PeerStore {
  peers = {};

  constructor() {
    makeAutoObservable(this);
  }

  setPeers(peers) {
    this.peers = peers;
  }

  setPeer(peerId, peer) {
    this.peers[peerId] = peer;
  }

  deletePeer(peerId) {
    if(!this.peers[peerId]) return;
    this.peers[peerId].destroy();
    delete this.peers[peerId];
  }
}

export default new PeerStore();