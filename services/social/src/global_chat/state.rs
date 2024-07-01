use std::collections::{HashMap, VecDeque};

use auth_server_types::UserId;
use social_server_types::{GlobalChatMessageId, Timestamp};

use crate::session_servers::SessionServerId;

pub struct GlobalChatState {
    chat_log: VecDeque<(GlobalChatMessageId, Timestamp, UserId, String)>,
    next_global_chat_id: GlobalChatMessageId,

    // the session server id here is the SENDER not the RECEIVER
    outgoing_patches:
        HashMap<SessionServerId, Vec<(GlobalChatMessageId, Timestamp, UserId, String)>>,
}

impl GlobalChatState {
    pub fn new() -> Self {
        Self {
            chat_log: VecDeque::new(),
            next_global_chat_id: GlobalChatMessageId::new(0),

            outgoing_patches: HashMap::new(),
        }
    }

    pub fn send_message(
        &mut self,
        sending_session_server_id: SessionServerId,
        user_id: UserId,
        message: &str,
    ) -> (GlobalChatMessageId, Timestamp) {
        // get next global chat id
        let next_global_chat_id = self.next_global_chat_id;
        self.next_global_chat_id = self.next_global_chat_id.next();

        // get timestamp
        let timestamp = Timestamp::now();

        // add to global log
        self.chat_log
            .push_back((next_global_chat_id, timestamp, user_id, message.to_string()));
        if self.chat_log.len() > 100 {
            self.chat_log.pop_front();
        }

        // add to outgoing patches
        if !self
            .outgoing_patches
            .contains_key(&sending_session_server_id)
        {
            self.outgoing_patches
                .insert(sending_session_server_id, Vec::new());
        }
        let session_server_patches = self
            .outgoing_patches
            .get_mut(&sending_session_server_id)
            .unwrap();
        session_server_patches.push((next_global_chat_id, timestamp, user_id, message.to_string()));

        (next_global_chat_id, timestamp)
    }

    pub fn get_full_log(&self) -> &VecDeque<(GlobalChatMessageId, Timestamp, UserId, String)> {
        &self.chat_log
    }

    pub fn take_patches(
        &mut self,
    ) -> HashMap<SessionServerId, Vec<(GlobalChatMessageId, Timestamp, UserId, String)>> {
        std::mem::take(&mut self.outgoing_patches)
    }
}
