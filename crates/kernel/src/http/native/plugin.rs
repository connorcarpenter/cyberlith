use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Update};

use crate::http::{CookieStore, HttpClient};

pub struct HttpClientPlugin {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl Default for HttpClientPlugin {
    fn default() -> Self {
        panic!("HttpClientPlugin::default() is not supported in native!");
    }
}

impl HttpClientPlugin {
    pub fn new(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self {
        Self { cookie_store_opt }
    }
}

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_core::TaskPoolPlugin>() {
            app.add_plugins(bevy_core::TaskPoolPlugin::default());
        }
        app.insert_resource(HttpClient::new(self.cookie_store_opt.clone()))
            .add_systems(Update, HttpClient::update_system);
    }
}