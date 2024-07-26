use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

// This is networked

#[derive(Component, Replicate)]
pub struct NextTilePosition {
    pub x: Property<i16>,
    pub y: Property<i16>,
}

impl NextTilePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_complete(x, y)
    }
}

// These are not networked

#[derive(Component)]
pub struct PrevTilePosition {
    pub x: i16,
    pub y: i16,
}

impl PrevTilePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn mirror(&mut self, other: &Self) {
        self.x = other.x;
        self.y = other.y;
    }
}

#[derive(Component)]
pub struct TileMovement {
    distance: f32,
    distance_max: f32,
    speed: f32,
}

impl TileMovement {
    pub fn new(speed: f32) -> Self {
        Self {
            distance: 0.0,
            distance_max: 1.0,
            speed,
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn distance_max(&self) -> f32 {
        self.distance_max
    }

    pub fn next(&mut self, new_distance: f32) {
        self.distance = 0.0;
        self.distance_max = new_distance;
    }

    pub fn tick(&mut self) {
        self.distance += self.speed;
        if self.distance >= self.distance_max {
            self.distance = self.distance_max;
        }
    }

    pub fn interp(&self) -> f32 {
        self.distance / self.distance_max
    }

    pub fn complete(&self) -> bool {
        self.distance >= self.distance_max
    }

    pub fn mirror(&mut self, other: &Self) {
        self.distance = other.distance;
        self.distance_max = other.distance_max;
        self.speed = other.speed;
    }
}

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn mirror(&mut self, other: &Self) {
        self.x = other.x;
        self.y = other.y;
    }
}