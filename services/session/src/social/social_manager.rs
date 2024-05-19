
use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct SocialManager {
    social_server_opt: Option<(String, u16)>,
}

impl SocialManager {
    pub fn new() -> Self {
        Self {
            social_server_opt: None,
        }
    }

    // Social Server

    pub fn set_social_server(&mut self, addr: &str, port: u16) {
        self.social_server_opt = Some((addr.to_string(), port));
    }

    pub fn clear_social_server(&mut self) {
        self.social_server_opt = None;
    }

    pub fn get_social_server_url(&self) -> Option<(String, u16)> {
        self.social_server_opt
            .as_ref()
            .map(|(addr, port)| (addr.clone(), *port))
    }
}
