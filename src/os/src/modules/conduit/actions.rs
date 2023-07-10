pub enum Action {
    DiscoverPeer(String),
    ForgetPeer(String),
    BlockPeer(String),
    RequestSendMessage(Message),
    ReceiveMessage(Packet),
    CancelRequest,
    CancelAllRequests,
    AdjustCongestionControlParameters(CongestionParams),
}
