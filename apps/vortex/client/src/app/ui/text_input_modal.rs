use bevy_ecs::{event::Events, world::World};
use bevy_log::info;

use egui_modal::Modal;

use render_egui::{
    egui,
    egui::{Align, Layout, Ui},
};

use crate::app::{
    config::AppConfig,
    events::LoginEvent,
    ui::{LoggingInState, UiState, utils::ui_with_margin},
};

pub type ModalRequestHandle = u16;

pub struct TextInputModal {
    open: bool,
    next_handle: ModalRequestHandle,
    current_handle: Option<ModalRequestHandle>,
    current_response: Option<String>,
    title: String,
    text: String,
    value: String,
    button_text: String,
}

impl TextInputModal {

    pub fn new() -> Self {
        Self {
            open: false,
            next_handle: 0,
            current_handle: None,
            current_response: None,
            title: String::new(),
            text: String::new(),
            value: String::new(),
            button_text: String::new(),
        }
    }

    pub fn open(&mut self, title: &str, text: &str, default_value: &str, button_text: &str) -> Option<ModalRequestHandle> {
        if self.open {
            return None;
        }

        self.open = true;
        self.title = title.to_string();
        self.text = text.to_string();
        self.value = default_value.to_string();
        self.button_text = button_text.to_string();

        self.current_handle = Some(self.next_handle);
        self.next_handle = self.next_handle.wrapping_add(1);
        return self.current_handle;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_response(&mut self, response: String) {
        self.current_response = Some(response);
    }

    pub fn take_response(&mut self, handle: ModalRequestHandle) -> Option<Option<String>> {
        if self.open {
            return None;
        }
        if self.current_handle != Some(handle) {
            return None;
        }

        let response = self.current_response.take();
        self.current_handle = None;
        return Some(response);
    }

    pub fn show(context: &egui::Context, world: &mut World) {

        let mut ui_state = world.get_resource_mut::<UiState>().unwrap();

        let mut modal_state = &mut ui_state.text_input_modal;

        if !modal_state.open {
            return;
        }

        let modal = Modal::new(context, "rename_modal").with_close_on_outside_click(true);

        let was_open = modal.is_open();
        if !was_open {
            // Just opened
            modal.open();
        }

        let margin = 5.0;

        modal.show(|ui| {
            modal.title(ui, &modal_state.title);
            modal.frame(ui, |ui| {

                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui_with_margin(ui, margin, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui_with_margin(ui, margin, |ui| {
                                ui.label(&modal_state.text);
                            })
                        });
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui_with_margin(ui, margin, |ui| {
                                ui.text_edit_singleline(&mut modal_state.value);
                            })
                        });
                    })
                });
            });
            modal.buttons(ui, |ui| {
                if modal.button(ui, "Cancel").clicked() {
                    // Cancel button clicked..
                }
                if modal.suggested_button(ui, &modal_state.button_text).clicked() {
                    modal_state.set_response(modal_state.value.clone());
                }
            });
        });

        if !modal.is_open() {
            // Just closed
            modal_state.close();
        }
    }
}

