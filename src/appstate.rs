use tokio::sync::broadcast;

pub struct AppState {
    tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(tx: broadcast::Sender<String>,) -> Self {
        Self{tx}
    }

    pub fn clone_tx(&self) -> broadcast::Sender<String> {
        self.tx.clone()
    }

    pub fn create_rx(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}
