use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{Query, Res, ResMut, SystemState},
    world::Mut,
};
use logging::info;

use naia_bevy_client::Client;

use editor_proto::components::{FaceColor, FileExtension};

use crate::app::{
    plugin::Main,
    resources::{
        action::{shape::entity_request_release, skin::SkinAction},
        canvas::Canvas,
        face_manager::FaceManager,
        file_manager::FileManager,
        input::InputManager,
        palette_manager::PaletteManager,
        shape_data::CanvasShape,
        skin_manager::SkinManager,
    },
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
        Client<Main>,
        ResMut<Canvas>,
        Res<FileManager>,
        Res<FaceManager>,
        Res<SkinManager>,
        Res<PaletteManager>,
        Query<&mut FaceColor>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        file_manager,
        face_manager,
        skin_manager,
        palette_manager,
        mut face_color_q,
    ) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) =
        deselect_selected_shape(&mut canvas, input_manager, &face_manager, &skin_manager);
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
        let face_3d_entity = face_manager.face_entity_2d_to_3d(&face_2d_entity).unwrap();

        let palette_color_index = skin_manager.selected_color_index();
        let Some(palette_file_entity) =
            file_manager.file_get_dependency(&current_file_entity, FileExtension::Palette)
        else {
            panic!("Expected palette file dependency");
        };
        let next_palette_color_entity = palette_manager
            .get_color_entity(&palette_file_entity, palette_color_index)
            .unwrap();

        if let Some(face_color_entity) = entity_to_request {
            // edit face color
            let Ok(mut face_color) = face_color_q.get_mut(face_color_entity) else {
                panic!(
                    "Failed to get FaceColor for face color entity {:?}!",
                    face_color_entity
                );
            };

            let prev_palette_entity = face_color.palette_color_entity.get(&client).unwrap();

            face_color
                .palette_color_entity
                .set(&client, &next_palette_color_entity);

            return vec![
                SkinAction::SelectFace(deselected_entity),
                SkinAction::EditColor(face_2d_entity, Some(prev_palette_entity)),
            ];
        } else {
            // create face color
            world.resource_scope(|world, mut skin_manager: Mut<SkinManager>| {
                skin_manager.create_networked_face_color_from_world(
                    world,
                    current_file_entity,
                    face_3d_entity,
                    next_palette_color_entity,
                );
            });

            system_state.apply(world);

            return vec![
                SkinAction::SelectFace(deselected_entity),
                SkinAction::EditColor(face_2d_entity, None),
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
            let face_3d_entity = face_manager.face_entity_2d_to_3d(&shape_2d_entity).unwrap();
            return get_color_entity(skin_manager, &face_3d_entity);
        }
        _ => {
            panic!("unexpected shape type");
        }
    }
}

fn get_color_entity(skin_manager: &SkinManager, face_3d_entity: &Entity) -> Option<Entity> {
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
                let face_3d_entity = face_manager.face_entity_2d_to_3d(&shape_2d_entity).unwrap();
                entity_to_release = get_color_entity(skin_manager, &face_3d_entity);
            }
            _ => {
                panic!("unexpected shape type");
            }
        }
    }
    (entity_to_deselect, entity_to_release)
}
