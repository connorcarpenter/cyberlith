use bevy_ecs::{event::Events, world::World};

use egui_modal::Modal;

use render_egui::{
    egui,
    egui::{Align, Layout},
};

use crate::app::{
    config::AppConfig,
    events::LoginEvent,
    ui::{utils::ui_with_margin, LoggingInState, UiState},
};

pub fn login_modal(context: &egui::Context, world: &mut World) {
    let mut creds: Option<(String, String)> = None;

    {
        // Pull login credentials from LoginConfig
        let config = world.get_resource::<AppConfig>().unwrap();
        if let Some(login_config) = &config.login {
            creds = Some((login_config.username.clone(), login_config.password.clone()));
        }
    }

    {
        let mut ui_state = world.get_resource_mut::<UiState>().unwrap();

        let modal = Modal::new(context, "login_modal");
        let margin = 5.0;

        modal.show(|ui| {
            modal.frame(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui_with_margin(ui, margin, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui_with_margin(ui, margin, |ui| {
                                ui.label("username: ");
                                if ui
                                    .text_edit_singleline(&mut ui_state.username)
                                    .gained_focus()
                                {
                                    ui_state.logging_in_state = LoggingInState::NotLoggingIn;
                                }
                            })
                        });
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui_with_margin(ui, margin, |ui| {
                                ui.label("password: ");
                                if ui
                                    .text_edit_singleline(&mut ui_state.password)
                                    .gained_focus()
                                {
                                    ui_state.logging_in_state = LoggingInState::NotLoggingIn;
                                }
                            })
                        });
                    })
                });
            });

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                match ui_state.logging_in_state {
                    LoggingInState::NotLoggingIn => {
                        // do nothing
                    }
                    LoggingInState::LoggingIn => {
                        ui.spinner();
                    }
                    LoggingInState::LoginFailed => {
                        ui.label("❌ invalid credentials ❌");
                    }
                }

                ui.separator();

                if modal.button(ui, "login").clicked() {
                    creds = Some((ui_state.username.clone(), ui_state.password.clone()));
                    ui_state.username = String::new();
                    ui_state.password = String::new();
                }
            });
        });
        modal.open();
    }

    {
        let ui_state = world.get_resource::<UiState>().unwrap();
        if ui_state.logging_in_state != LoggingInState::LoggingIn {
            if let Some((username, password)) = creds {
                let mut login_events = world.get_resource_mut::<Events<LoginEvent>>().unwrap();
                login_events.send(LoginEvent { username, password });

                let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                ui_state.logging_in_state = LoggingInState::LoggingIn;
            }
        }
    }
}
