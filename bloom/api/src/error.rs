#[derive(Debug)]
pub enum BloomError {
    // Configuration
    ProjectNotFound = 100,
    InvalidProject = 101,

    // Process
    WaitOnChildFailed = 200,
    KillChildFailed = 201,
    SpawnChildFailed = 202,

    // Networking
    FindPortFailed = 300,
    ConnectToFrontendFailed = 301,
    ChannelRecieveError = 302,
    ChannelSendError = 303,
    TcpBindFailed = 304,
    TcpAcceptFailed = 305,
    TcpStreamCloneFailed = 306,
    TcpReadFailed = 307,
    TcpWriteFailed = 308,

    // Protocol
    PacketTooLarge = 400,
    InvalidPacketSize = 401,
    PacketNotTerminated = 402,
    InvalidClientId = 403,
    InvalidAppName = 404,
    AppNameTooLong = 405,
    InvalidPacket = 406,
    InvalidMessageUse = 407,
    ClientConnectionFailed = 408,
    FailedToObtainClientStream = 409,
    FailedToObtainClientDisconnectChannel = 410,
    ClientDisconnected = 411,
}
