use bevy_ecs::{
    entity::Entity,
    system::{Commands, SystemState},
    world::World,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use render_egui::{
    egui,
    egui::{Button, Direction, Frame, Layout, Ui},
};

use vortex_proto::components::FileDependency;

use crate::app::{
    resources::file_manager::FileManager,
    ui::{BindingState, UiState},
};

pub fn render_bind_button(ui: &mut Ui, world: &mut World, current_file_entity: &Entity) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
            Frame::none().inner_margin(300.0).show(ui, |ui| {
                let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                match ui_state.binding_file {
                    BindingState::NotBinding => {
                        if ui.button("Bind to .skel File").clicked() {
                            ui_state.binding_file = BindingState::Binding;
                        }
                    }
                    BindingState::Binding => {
                        ui.add_enabled(
                            false,
                            Button::new("Click on .skel file in sidebar to bind."),
                        );
                    }
                    BindingState::BindResult(dependency_file_entity) => {
                        let mut file_manager = world.get_resource_mut::<FileManager>().unwrap();

                        info!("received bind result for dependency");

                        file_manager
                            .file_add_dependency(&current_file_entity, &dependency_file_entity);

                        let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                        ui_state.binding_file = BindingState::NotBinding;

                        // send message to server
                        let mut system_state: SystemState<(Commands, Client)> =
                            SystemState::new(world);
                        let (mut commands, mut client) = system_state.get_mut(world);

                        let mut component = FileDependency::new();
                        component.file_entity.set(&client, &current_file_entity);
                        component
                            .dependency_entity
                            .set(&client, &dependency_file_entity);
                        let dependency_entity = commands
                            .spawn_empty()
                            .enable_replication(&mut client)
                            .configure_replication(ReplicationConfig::Delegated)
                            .insert(component)
                            .id();

                        system_state.apply(world);

                        let mut system_state: SystemState<(Commands, Client)> =
                            SystemState::new(world);
                        let (mut commands, mut client) = system_state.get_mut(world);

                        commands
                            .entity(dependency_entity)
                            .release_authority(&mut client);

                        system_state.apply(world);
                    }
                }
            });
        });
    });
}
