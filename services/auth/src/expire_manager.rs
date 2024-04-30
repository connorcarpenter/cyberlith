use std::time::Duration;

use timequeue::TimeQueue;
use instant::Instant;

use crate::types::{AccessToken, RefreshToken, RegisterToken, ResetPasswordToken};

#[derive(Eq, PartialEq)]
pub(crate) enum ExpireEvent {
    AccessToken(AccessToken),
    RefreshToken(RefreshToken),
    RegisterToken(RegisterToken),
    ResetPasswordToken(ResetPasswordToken),
}

pub struct ExpireManager {
    time_queue: TimeQueue<ExpireEvent>
}

impl ExpireManager {
    pub fn new() -> Self {
        Self {
            time_queue: TimeQueue::new()
        }
    }

    pub fn insert_access_token(&mut self, token: &AccessToken) {
        let mut now = Instant::now();
        let duration = Duration::from_secs(60 * 60); // 1 hour // TODO: move into config var
        now.add_millis(duration.as_millis() as u32);
        self.time_queue.add_item(now, ExpireEvent::AccessToken(*token));
    }

    pub fn insert_refresh_token(&mut self, token: &RefreshToken) {
        let mut now = Instant::now();
        let duration = Duration::from_secs(60 * 60 * 24 * 7); // 1 week // TODO: move into config var
        now.add_millis(duration.as_millis() as u32);
        self.time_queue.add_item(now, ExpireEvent::RefreshToken(*token));
    }

    pub fn insert_register_token(&mut self, token: &RegisterToken) {
        let mut now = Instant::now();
        let duration = Duration::from_secs(60 * 60 * 24); // 1 day // TODO: move into config var
        now.add_millis(duration.as_millis() as u32);
        self.time_queue.add_item(now, ExpireEvent::RegisterToken(*token));
    }

    pub fn insert_reset_password_token(&mut self, token: &ResetPasswordToken) {
        let mut now = Instant::now();
        let duration = Duration::from_secs(60 * 60 * 24); // 1 day // TODO: move into config var
        now.add_millis(duration.as_millis() as u32);
        self.time_queue.add_item(now, ExpireEvent::ResetPasswordToken(*token));
    }

    pub fn clear_expired(&mut self) -> Vec<ExpireEvent> {
        let mut output = Vec::new();
        let now = Instant::now();
        while self.time_queue.has_item(&now) {
            let event = self.time_queue.pop_item(&now).unwrap();
            output.push(event);
        }
        output
    }
}