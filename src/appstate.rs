use std::{collections::HashMap, net::SocketAddr};

use tokio::{sync::{broadcast, Mutex}, task::JoinHandle};

pub struct AppState {
    tx: broadcast::Sender<String>,
    calc_tasks: Mutex<HashMap<SocketAddr, JoinHandle<()>>>,
}

impl AppState {
    pub fn new(tx: broadcast::Sender<String>,) -> Self {
        let calc_tasks = Mutex::new(HashMap::new());
        Self{tx, calc_tasks}
    }

    pub fn clone_tx(&self) -> broadcast::Sender<String> {
        self.tx.clone()
    }

    pub fn create_rx(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    // TODO: one method for insert and abort?
    pub async fn add(&self, who: SocketAddr, task: JoinHandle<()>) {
        let mut lock = self.calc_tasks.lock().await;
        if let Some(old_task) = lock.insert(who, task){
            old_task.abort();
        }
    }

    pub async fn abort_if_exists(&self, who: &SocketAddr) {
        let mut lock = self.calc_tasks.lock().await;
        if let Some(task) = lock.remove(who) {
            task.abort();
            tracing::debug!("calc aborted");
        }
    }
}
