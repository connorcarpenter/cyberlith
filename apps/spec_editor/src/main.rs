mod catalog;
mod writers;

mod animated_models;
mod movement_configs;
mod units;

use writers::{
    animated_model::write_to_file as write_animated_model,
    movement_config::write_to_file as write_movement_config, unit::write_to_file as write_unit,
};

fn main() {
    // avatar.animated_model
    write_animated_model(animated_models::avatar::define());

    // avatar.movement_config
    write_movement_config(movement_configs::avatar::define());

    // avatar.unit
    write_unit(units::avatar::define());
}
