use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate, Tick};

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
    predicted: bool,
    tick: Tick,
    distance: f32,
    distance_max: f32,
    speed: f32,
    done: bool
}

impl TileMovement {
    pub fn new(predicted: bool, tick: Tick, speed: f32) -> Self {
        Self {
            predicted,
            tick,
            distance: 0.0,
            distance_max: 1.0,
            speed,
            done: false,
        }
    }

    // pub fn tick(&self) -> Tick {
    //     self.tick
    // }

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
        self.done = false;
    }

    pub fn process_tick(&mut self, tick: Tick) {
        self.tick = tick;
        if self.done {
            return;
        }
        self.distance += self.speed;
        if self.distance >= self.distance_max {
            self.done = true;
            self.distance = self.distance_max;
        }
    }

    pub fn interp(&self) -> f32 {
        if self.done {
            return 1.0;
        }
        return self.distance / self.distance_max;
    }

    pub fn complete(&self) -> bool {
        self.done
    }

    pub fn mirror(&mut self, other: &Self) {
        self.tick = other.tick;
        self.distance = other.distance;
        self.distance_max = other.distance_max;
        self.speed = other.speed;
        self.done = other.done;
    }
}

#[derive(Component, Debug)]
pub struct Position {
    predicted: bool,
    tick: Tick,
    x: f32,
    y: f32,
}

impl Position {
    pub fn new(predicted: bool, tick: Tick, x: f32, y: f32) -> Self {
        Self { predicted, tick, x, y }
    }

    pub fn predicted(&self) -> bool {
        self.predicted
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn set(&mut self, tick: Tick, x: f32, y: f32) {
        self.tick = tick;
        self.x = x;
        self.y = y;
    }

    pub fn mirror(&mut self, other: &Self) {
        self.tick = other.tick;
        self.x = other.x;
        self.y = other.y;
    }
}