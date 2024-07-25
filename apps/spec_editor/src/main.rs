
mod catalog;
mod animated_models;
mod movement_configs;
mod writers;

use writers::{
    movement_config::write_to_file as write_movement_config,
    animated_model::write_to_file as write_animated_model,
};

fn main() {
    // avatar.animated_model
    write_animated_model(
        animated_models::avatar::define()
    );

    // avatar.movement_config
    write_movement_config(
        movement_configs::avatar::define()
    );
}