use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{Res, ResMut, SystemState},
    world::Mut,
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::FileExtension;

use crate::app::resources::{
    palette_manager::PaletteManager,
    file_manager::FileManager,
    face_manager::FaceManager,
    skin_manager::SkinManager,
    action::{skin::SkinAction, shape::entity_request_release},
    canvas::Canvas,
    input_manager::InputManager,
    shape_data::CanvasShape,
};

pub fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    current_file_entity: Entity,
    action: SkinAction,
) -> Vec<SkinAction> {
    let SkinAction::SelectFace(shape_2d_entity_opt) = action else {
        panic!("Expected SelectFace");
    };

    info!("SkinSelectFace({:?})", shape_2d_entity_opt);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        Res<FileManager>,
        Res<FaceManager>,
        Res<SkinManager>,
        Res<PaletteManager>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        file_manager,
        face_manager,
        skin_manager,
        palette_manager,
    ) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) = deselect_selected_shape(
        &mut canvas,
        input_manager,
        &face_manager,
        &skin_manager,
    );
    let entity_to_request = select_shape(
        &mut canvas,
        input_manager,
        &face_manager,
        &skin_manager,
        shape_2d_entity_opt,
    );
    entity_request_release(
        &mut commands,
        &mut client,
        entity_to_request,
        entity_to_release,
    );



    if let Some((face_2d_entity, CanvasShape::Face)) = shape_2d_entity_opt {
        if entity_to_request.is_none() {

            let face_3d_entity = face_manager
                .face_entity_2d_to_3d(&face_2d_entity)
                .unwrap();

            let palette_color_index = skin_manager.selected_color_index();
            let Some(palette_file_entity) = file_manager.file_get_dependency(
                &current_file_entity,
                FileExtension::Palette,
            ) else {
                panic!("Expected palette file dependency");
            };

            let next_palette_color_entity = palette_manager.get_color_entity(&palette_file_entity, palette_color_index).unwrap();

            world.resource_scope(|world, mut skin_manager: Mut<SkinManager>| {
                skin_manager.create_networked_face_color_from_world(world, face_3d_entity, next_palette_color_entity);
            });

            system_state.apply(world);

            return vec![
                SkinAction::SelectFace(deselected_entity),
                SkinAction::EditColor(face_2d_entity, Some(next_palette_color_entity), None),
            ];
        }
    }

    system_state.apply(world);

    return vec![SkinAction::SelectFace(deselected_entity)];
}

// returns entity to request auth for
fn select_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    face_manager: &FaceManager,
    skin_manager: &SkinManager,
    shape_2d_entity_opt: Option<(Entity, CanvasShape)>,
) -> Option<Entity> {
    let (shape_2d_entity, shape) = shape_2d_entity_opt?;
    input_manager.select_shape(canvas, &shape_2d_entity, shape);

    match shape {
        CanvasShape::Face => {
            let face_3d_entity = face_manager
                .face_entity_2d_to_3d(&shape_2d_entity)
                .unwrap();
            return get_color_entity(skin_manager, face_3d_entity);
        }
        _ => {
            panic!("unexpected shape type");
        }
    }
}

fn get_color_entity(
    skin_manager: &SkinManager,
    face_3d_entity: Entity,
) -> Option<Entity> {
    return skin_manager
        .face_to_color_entity(face_3d_entity)
        .map(|entity| *entity);
}

fn deselect_selected_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    face_manager: &FaceManager,
    skin_manager: &SkinManager,
) -> (Option<(Entity, CanvasShape)>, Option<Entity>) {
    let mut entity_to_deselect = None;
    let mut entity_to_release = None;
    if let Some((shape_2d_entity, shape_2d_type)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(canvas);
        entity_to_deselect = Some((shape_2d_entity, shape_2d_type));

        match shape_2d_type {
            CanvasShape::Face => {
                let face_3d_entity = face_manager
                    .face_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                entity_to_release =
                    get_color_entity(skin_manager, face_3d_entity);
            }
            _ => {
                panic!("unexpected shape type");
            }
        }
    }
    (entity_to_deselect, entity_to_release)
}
