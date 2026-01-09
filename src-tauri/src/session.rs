use std::collections::HashMap;
use tokio::sync::oneshot;

pub struct TailSessionManager {
    sessions: HashMap<String, oneshot::Sender<()>>,
}

impl TailSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn add_session(&mut self, session_id: String, stopper: oneshot::Sender<()>) {
        if let Some(old_stopper) = self.sessions.remove(&session_id) {
            let _ = old_stopper.send(());
        }
        self.sessions.insert(session_id, stopper);
    }

    pub fn stop_session(&mut self, session_id: &str) {
        if let Some(stopper) = self.sessions.remove(session_id) {
            let _ = stopper.send(());
        }
    }

    pub fn stop_all(&mut self) {
        for (_, stopper) in self.sessions.drain() {
            let _ = stopper.send(());
        }
    }
}
