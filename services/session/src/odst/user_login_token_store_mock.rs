use std::collections::HashSet;

use auth_server_types::UserId;

pub(crate) struct UserLoginTokenStore {
    user_ids: HashSet<UserId>,
    next_user_id: UserId,
}

impl UserLoginTokenStore {
    pub fn new() -> Self {
        Self {
            user_ids: HashSet::new(),
            next_user_id: UserId::new(1),
        }
    }

    pub fn recv_login_token(&mut self, _user_id: &UserId, _token: &str) {
        panic!("ODST mode does not support this function");
    }

    pub fn spend_login_token(&mut self, token: &str) -> Option<UserId> {
        if token.eq_ignore_ascii_case("odst") {
            let user_id = self.next_user_id;
            let user_id_u64: u64 = user_id.into();
            self.next_user_id = UserId::new(user_id_u64 + 1);

            self.user_ids.insert(user_id);

            Some(user_id)
        } else {
            None
        }
    }

    // TODO: recycle user ids? via disconnect handler?
}
