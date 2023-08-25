use std::collections::HashMap;
use bevy_ecs::entity::Entity;

use bevy_ecs::system::Resource;
use bevy_log::{info, warn};
use naia_bevy_server::{RoomKey, Server, UserKey};
use vortex_proto::resources::FileEntryKey;
use vortex_proto::types::TabId;

use crate::resources::project::ProjectKey;
use crate::resources::{ContentEntityData, FileSpace, GitManager, UserTabState};

pub struct UserSessionData {
    // used to index into permanent data
    username: String,
    // current project data
    project_owner_name: String,
    project_key: Option<ProjectKey>,
    // tabs
    tab_state: UserTabState,
}

impl UserSessionData {
    pub fn new(username: &str, project_owner_name: &str) -> Self {
        Self {
            username: username.to_string(),
            project_owner_name: project_owner_name.to_string(),
            project_key: None,
            tab_state: UserTabState::default(),
        }
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn project_owner_name(&self) -> &str {
        &self.project_owner_name
    }

    pub(crate) fn project_key(&self) -> Option<ProjectKey> {
        self.project_key
    }

    pub(crate) fn set_project_key(&mut self, project_key: ProjectKey) {
        self.project_key = Some(project_key);
    }

    pub(crate) fn tab_state(&self) -> &UserTabState {
        &self.tab_state
    }

    pub(crate) fn tab_state_mut(&mut self) -> &mut UserTabState {
        &mut self.tab_state
    }

    pub(crate) fn current_tab(&self) -> Option<TabId> {
        self.tab_state.current_tab()
    }

    pub(crate) fn current_tab_file_key(&self) -> Option<FileEntryKey> {
        if let Some(current_tab) = self.tab_state.current_tab() {
            self.tab_state.tab_file_key(&current_tab)
        } else {
            None
        }
    }

    pub(crate) fn open_tab(&mut self, tab_id: TabId, file_key: FileEntryKey) {
        self.tab_state.insert_tab(tab_id, file_key);
    }

    pub(crate) fn close_tab(&mut self, tab_id: &TabId) -> Option<FileEntryKey> {
        if self.tab_state.current_tab() == Some(tab_id.clone()) {
            self.tab_state.set_current_tab(None);
        }
        self.tab_state.remove_tab(tab_id)
    }
}

pub struct UserPermanentData {
    username: String,
    email: String,
    password: String,
    // this should be toggleable later, so no need for it here
    starting_project_owner_name: String,
}

impl UserPermanentData {
    pub fn new(username: &str, email: &str, password: &str, starting_project_owner_name: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            starting_project_owner_name: starting_project_owner_name.to_string(),
        }
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Resource)]
pub struct UserManager {
    // HashMap<username, UserPermanentData>
    user_permanent_data: HashMap<String, UserPermanentData>,
    user_sessions: HashMap<UserKey, UserSessionData>,
}

impl Default for UserManager {
    fn default() -> Self {
        let mut credentials = HashMap::new();

        // Connor
        credentials.insert(
            "connorcarpenter".to_string(),
            UserPermanentData::new("connorcarpenter", "connorcarpenter@gmail.com", "greattobealive!", "connorcarpenter"),
        );

        // Brendon?
        credentials.insert(
            "brendoncarpenter".to_string(),
            UserPermanentData::new("brendoncarpenter", "brendon.e.carpenter@gmail.com", "greattobealive!", "connorcarpenter"),
        );

        // TODO: add more users here? get from database?

        Self {
            user_permanent_data: credentials,
            user_sessions: HashMap::new(),
        }
    }
}

impl UserManager {
    pub fn validate_user(&self, username: &str, password: &str) -> bool {
        match self.user_permanent_data.get(username) {
            Some(user_data) => {
                if user_data.password == password {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn user_perm_data(&self, user_key: &UserKey) -> Option<&UserPermanentData> {
        let Some(user_session_data) = self.user_sessions.get(user_key) else {
            return None;
        };
        let username = user_session_data.username();
        self.user_permanent_data.get(username)
    }

    pub fn user_session_data(&self, user_key: &UserKey) -> Option<&UserSessionData> {
        self.user_sessions.get(user_key)
    }

    pub fn user_session_data_mut(&mut self, user_key: &UserKey) -> Option<&mut UserSessionData> {
        self.user_sessions.get_mut(user_key)
    }

    pub(crate) fn user_tab_state_mut(&mut self, user_key: &UserKey) -> Option<&mut UserTabState> {
        let Some(user_session) = self.user_sessions.get_mut(user_key) else {
            panic!("user not found");
        };
        Some(user_session.tab_state_mut())
    }

    pub fn login_user(&mut self, user_key: &UserKey, user_name: &str) {
        let Some(user_data) = self.user_permanent_data.get(user_name) else {
            panic!("user not found");
        };
        let project_owner_name = user_data.starting_project_owner_name.clone();
        self.user_sessions.insert(*user_key, UserSessionData::new(user_name, &project_owner_name));
    }

    pub fn logout_user(&mut self, user_key: &UserKey) {
        self.user_sessions.remove(user_key);
    }

    pub(crate) fn open_tab(&mut self, user_key: &UserKey, tab_id: TabId, file_key: FileEntryKey) {
        let Some(user_session) = self.user_sessions.get_mut(user_key) else {
            panic!("user not found");
        };
        user_session.open_tab(tab_id, file_key);
    }

    pub(crate) fn close_tab(&mut self, server: &mut Server, git_manager: &mut GitManager, user_key: &UserKey, tab_id: &TabId) -> (ProjectKey, FileEntryKey, HashMap<Entity, ContentEntityData>) {

        let Some(user_session) = self.user_sessions.get_mut(user_key) else {
            panic!("User does not exist!");
        };
        let Some(file_key) = user_session.close_tab(tab_id) else {
            panic!("User tab does not exist");
        };

        let Some(project_key) = user_session.project_key() else {
            panic!("User does not have project key");
        };
        let project = git_manager.project_mut(&project_key).unwrap();
        let content_entities = project.user_leave_filespace(server, &file_key);

        (project_key, file_key, content_entities)
    }

    pub fn select_tab(
        &mut self,
        user_key: &UserKey,
        tab_id: &TabId,
    ) {
        let Some(user_tab_state) = self.user_tab_state_mut(user_key) else {
            panic!("user does not exist")
        };
        if !user_tab_state.has_tab_id(tab_id) {
            warn!("User does not have tab {}", tab_id);
            return;
        }

        info!("Select Tab!");

        // Switch current Tab
        user_tab_state.set_current_tab(Some(tab_id.clone()));
    }
}
