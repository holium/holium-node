pub enum Effect {
    PeerDiscovered,
    PeerForgotten,
    PeerBlocked,
    SendMessage,
    MessageReceived,
    CongestionControlParametersAdjusted,
    CrashOnMessageSend,
    CrashOnMessageReceived,
    CrashOnDiscovery,
}
