pub enum BloomError {
    // Configuration
    ProjectNotFound = 100,
    InvalidProject = 101,

    // Process
    WaitOnChildFailed,
    KillChildFailed,
    SpawnChildFailed,

    // Networking
    FindPortFailed,
    ConnectToFrontendFailed,
}
