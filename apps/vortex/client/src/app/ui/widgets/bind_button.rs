use bevy_ecs::entity::Entity;
use bevy_ecs::system::{ResMut, SystemState};
use bevy_ecs::world::World;
use naia_bevy_client::Client;

use render_egui::{egui, egui::{Button, Direction, Frame, Layout, Ui}};
use vortex_proto::channels::FileActionChannel;
use vortex_proto::components::{FileSystemEntry, FileTypeValue};
use vortex_proto::messages::FileBindMessage;

use crate::app::{resources::{file_manager::FileManager}, ui::{BindingState, UiState}};

pub fn render_bind_button(ui: &mut Ui, world: &mut World, current_file_entity: Entity) {
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                Frame::none()
                    .inner_margin(300.0)
                    .show(ui, |ui| {
                        let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                        match ui_state.binding_file {
                            BindingState::NotBinding => {
                                if ui.button("Bind to .skel File").clicked() {
                                    ui_state.binding_file = BindingState::Binding;
                                }
                            }
                            BindingState::Binding => {
                                ui.add_enabled(false, Button::new("Click on .skel file in sidebar to bind."));
                            }
                            BindingState::BindResult(dependency_file_entity) => {

                                let mut file_manager = world.get_resource_mut::<FileManager>().unwrap();
                                file_manager.file_add_dependency(&current_file_entity, &dependency_file_entity);

                                let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                                ui_state.binding_file = BindingState::NotBinding;

                                // send message to server
                                let mut system_state: SystemState<Client> = SystemState::new(world);
                                let mut client = system_state.get_mut(world);
                                let message = FileBindMessage::new(&client, &current_file_entity, &dependency_file_entity);
                                client.send_message::<FileActionChannel, FileBindMessage>(&message);
                            }
                        }
                    });
            });
        });
}

pub fn render_bound(ui: &mut Ui, world: &mut World, current_file_entity: Entity) {
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                Frame::none()
                    .inner_margin(300.0)
                    .show(ui, |ui| {
                        let file_manager = world.get_resource::<FileManager>().unwrap();
                        let dependency_entity = file_manager.file_get_dependency(current_file_entity, FileTypeValue::Skel).unwrap();
                        let dependency_name = world.query::<&FileSystemEntry>().get(world, dependency_entity).unwrap().name.as_str();
                        ui.add_enabled(false, Button::new(dependency_name));
                    });
            });
        });
}