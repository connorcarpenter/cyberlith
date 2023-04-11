use bevy_ecs::{event::Events, world::World};

use egui_modal::Modal;

use render_egui::{
    egui,
    egui::{Align, Layout, Ui},
};

use crate::app::{events::LoginEvent, ui::{UiState, LoggingInState}};

pub fn login_modal(context: &egui::Context, world: &mut World) {
    let mut creds: Option<(String, String)> = None;

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
                                if ui.text_edit_singleline(&mut ui_state.username).gained_focus() {
                                    ui_state.logging_in_state = LoggingInState::NotLoggingIn;
                                }
                            })
                        });
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui_with_margin(ui, margin, |ui| {
                                ui.label("password: ");
                                if ui.text_edit_singleline(&mut ui_state.password).gained_focus() {
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

    if let Some((username, password)) = creds {
        let mut login_events = world.get_resource_mut::<Events<LoginEvent>>().unwrap();
        login_events.send(LoginEvent { username, password });

        let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
        ui_state.logging_in_state = LoggingInState::LoggingIn;
    }
}

fn ui_with_margin<R>(ui: &mut Ui, margin: f32, add_contents: impl FnOnce(&mut Ui) -> R) {
    egui::Frame::none()
        .inner_margin(margin)
        .show(ui, |ui| add_contents(ui));
}
