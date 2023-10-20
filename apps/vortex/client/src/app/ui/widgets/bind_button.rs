use std::collections::HashSet;

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

use vortex_proto::components::{FileDependency, FileExtension};

use crate::app::{
    resources::file_manager::FileManager,
    ui::{BindingState, UiState},
};

pub fn render_bind_button(
    ui: &mut Ui,
    world: &mut World,
    file_exts: &[FileExtension],
) -> Option<(FileExtension, Entity)> {
    let file_ext_str = get_ext_reqs_string(file_exts);
    let mut init_binding = false;
    let mut result = None;

    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
            Frame::none().inner_margin(300.0).show(ui, |ui| {
                match &world.get_resource::<UiState>().unwrap().binding_file {
                    BindingState::NotBinding => {
                        if ui
                            .button(format!("Bind to {} File", file_ext_str))
                            .clicked()
                        {
                            init_binding = true;
                        }
                    }
                    BindingState::Binding(_ext_req) => {
                        ui.add_enabled(
                            false,
                            Button::new(format!(
                                "Click on {} File in sidebar to bind.",
                                file_ext_str
                            )),
                        );
                    }
                    BindingState::BindResult(dependency_file_ext, dependency_file_entity) => {
                        info!("received bind result for dependency");
                        result = Some((*dependency_file_ext, *dependency_file_entity));
                    }
                };
            });
        });
    });

    if init_binding {
        let mut exts_set = HashSet::new();
        for ext in file_exts {
            exts_set.insert(*ext);
        }
        world.get_resource_mut::<UiState>().unwrap().binding_file = BindingState::Binding(exts_set);
    }

    return result;
}

pub fn render_bind_button_result(
    world: &mut World,
    current_file_entity: &Entity,
    dependency_file_entity: &Entity,
) {
    world.get_resource_mut::<UiState>().unwrap().binding_file = BindingState::NotBinding;
    create_networked_dependency(world, current_file_entity, &dependency_file_entity);
}

pub fn create_networked_dependency(
    world: &mut World,
    current_file_entity: &Entity,
    dependency_file_entity: &Entity,
) {
    // send message to server
    let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
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

    let mut file_manager = world.get_resource_mut::<FileManager>().unwrap();
    file_manager.file_add_dependency(
        &dependency_entity,
        &current_file_entity,
        &dependency_file_entity,
    );

    let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
    let (mut commands, mut client) = system_state.get_mut(world);

    commands
        .entity(dependency_entity)
        .release_authority(&mut client);

    system_state.apply(world);
}

fn get_ext_reqs_string(exts: &[FileExtension]) -> String {
    let mut output = String::new();

    let mut had_one = false;
    for ext in exts.iter() {
        if had_one {
            output.push_str("/");
        }
        output.push_str(&ext.to_string());
        had_one = true;
    }

    output
}
