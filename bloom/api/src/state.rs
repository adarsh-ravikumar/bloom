use crate::{AppChannel, BloomConnection, channel::AppEvent};

use std::sync::{Mutex, OnceLock};

static GLOBAL_STATE: OnceLock<GlobalState> = OnceLock::new();

#[derive(Debug)]
pub struct GlobalState {
    pub client: Mutex<Option<BloomConnection>>,
    pub app_channel: AppChannel,
}

pub fn init(state: GlobalState) {
    GLOBAL_STATE
        .set(state)
        .expect("Global state already initialized");
}

pub fn global_state() -> &'static GlobalState {
    GLOBAL_STATE.get().expect("Global state not initialized")
}

impl GlobalState {
    pub fn new(app_channel: AppChannel) -> Self {
        Self {
            client: Mutex::new(None),
            app_channel,
        }
    }

    pub fn set_client(&self, client: Option<BloomConnection>) {
        self.app_channel.send(AppEvent::SetClient(client));
    }

    pub fn get_client(&self) -> Option<BloomConnection> {
        self.client.lock().unwrap().clone()
    }

    pub fn terminate_app(&self) {
        self.app_channel.send(AppEvent::Terminate);
    }
}
