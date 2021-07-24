use lazy_static::lazy_static;
use ndarray::prelude::*;
use ndarray_stats::errors::MinMaxError;
use ndarray_stats::QuantileExt;

/// On kickoff
static SPAWN_COORDS: [[f32; 2]; 10] = [
    // Blue
    [-2048.0, -2560.0],
    [2048.0, -2560.0],
    [-256.0, -3840.0],
    [256.0, -3840.0],
    [0.0, -4608.0],
    // Orange
    [-2048.0, 2560.0],
    [2048.0, 2560.0],
    [-256.0, 3840.0],
    [256.0, 3840.0],
    [0.0, 4608.0],
];

/// From demolitions
static RESPAWN_LOCATION: [[f32; 2]; 8] = [
    // Blue
    [-2304.0, -4608.0],
    [-2688.0, -4608.0],
    [2304.0, -4608.0],
    [2688.0, -4608.0],
    // Orange
    [-2304.0, 4608.0],
    [-2688.0, 4608.0],
    [2304.0, 4608.0],
    [2688.0, 4608.0],
];

lazy_static! {
    pub static ref SPAWN_DISTANCE_CALCULATOR: SpawnDistanceCalculator =
        SpawnDistanceCalculator::new();
}

pub struct SpawnDistanceCalculator {
    spawns: Array2<f32>,
    respawns: Array2<f32>,
    distance_buffer: f32,
}

impl SpawnDistanceCalculator {
    pub fn new() -> Self {
        Self {
            spawns: arr2(&SPAWN_COORDS),
            respawns: arr2(&RESPAWN_LOCATION),
            distance_buffer: 5.0,
        }
    }

    pub fn check_if_near_spawn(&self, x: f32, y: f32) -> Result<bool, MinMaxError> {
        let spawn_minimum_distance = self.calculate_minimum_spawn_distance(x, y)?;
        let respawn_minimum_distance = self.calculate_minimum_respawn_distance(x, y)?;
        Ok(spawn_minimum_distance < self.distance_buffer
            || respawn_minimum_distance < self.distance_buffer)
    }

    pub fn calculate_minimum_spawn_distance(&self, x: f32, y: f32) -> Result<f32, MinMaxError> {
        let position = arr2(&[[x, y]]);
        let spawn_distances = (&self.spawns - &position)
            .mapv(|a| a * a)
            .sum_axis(Axis(1))
            .mapv(f32::sqrt);
        Ok(*spawn_distances.min()?)
    }

    pub fn calculate_minimum_respawn_distance(&self, x: f32, y: f32) -> Result<f32, MinMaxError> {
        let position = arr2(&[[x, y]]);
        let respawn_distances = (&self.respawns - &position)
            .mapv(|a| a * a)
            .sum_axis(Axis(1))
            .mapv(f32::sqrt);
        Ok(*respawn_distances.min()?)
    }
}

impl Default for SpawnDistanceCalculator {
    fn default() -> Self {
        SpawnDistanceCalculator::new()
    }
}
