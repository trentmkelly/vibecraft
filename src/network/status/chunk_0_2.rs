use super::*;

impl Drop for ActiveLoginGuard {
    fn drop(&mut self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            if sessions
                .get(&self.uuid)
                .is_some_and(|session| session.token == self.token)
            {
                sessions.remove(&self.uuid);
            }
        }
    }
}
