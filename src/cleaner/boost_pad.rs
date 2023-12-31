use std::sync::OnceLock;

use crate::cleaner::BoostPickupKind;
use log::warn;
use ndarray::prelude::*;
use ndarray_stats::errors::MinMaxError;
use ndarray_stats::QuantileExt;

pub static BOOST_PADS_COORDS: [[f32; 3]; 34] = [
    [0.0, -4240.0, 70.0],
    [-1792.0, -4184.0, 70.0],
    [1792.0, -4184.0, 70.0],
    [-3072.0, -4096.0, 73.0],
    [3072.0, -4096.0, 73.0],
    [-940.0, -3308.0, 70.0],
    [940.0, -3308.0, 70.0],
    [0.0, -2816.0, 70.0],
    [-3584.0, -2484.0, 70.0],
    [3584.0, -2484.0, 70.0],
    [-1788.0, -2300.0, 70.0],
    [1788.0, -2300.0, 70.0],
    [-2048.0, -1036.0, 70.0],
    [0.0, -1024.0, 70.0],
    [2048.0, -1036.0, 70.0],
    [-3584.0, 0.0, 73.0],
    [-1024.0, 0.0, 70.0],
    [1024.0, 0.0, 70.0],
    [3584.0, 0.0, 73.0],
    [-2048.0, 1036.0, 70.0],
    [0.0, 1024.0, 70.0],
    [2048.0, 1036.0, 70.0],
    [-1788.0, 2300.0, 70.0],
    [1788.0, 2300.0, 70.0],
    [-3584.0, 2484.0, 70.0],
    [3584.0, 2484.0, 70.0],
    [0.0, 2816.0, 70.0],
    [-940.0, 3310.0, 70.0],
    [940.0, 3308.0, 70.0],
    [-3072.0, 4096.0, 73.0],
    [3072.0, 4096.0, 73.0],
    [-1792.0, 4184.0, 70.0],
    [1792.0, 4184.0, 70.0],
    [0.0, 4240.0, 70.0],
];

pub static FULL_BOOST_PAD_RADIUS: f32 = 208.0;
pub static SMALL_BOOST_PAD_RADIUS: f32 = 144.0;
// See https://www.youtube.com/watch?v=xgfa-qZyInw for more details regarding boost pads
// In particular, waiting on a boost pad triggers a different (larger, square) hitbox.

fn full_boost_pads() -> Vec<[f32; 3]> {
    static FULL_BOOST_PADS: OnceLock<Vec<[f32; 3]>> = OnceLock::new();

    FULL_BOOST_PADS
        .get_or_init(|| {
            BOOST_PADS_COORDS
                .iter()
                .filter(|coords| (coords[2] - 73.0).abs() < f32::EPSILON)
                .cloned()
                .collect()
        })
        .to_vec()
}

fn small_boost_pads() -> Vec<[f32; 3]> {
    static SMALL_BOOST_PADS: OnceLock<Vec<[f32; 3]>> = OnceLock::new();
    SMALL_BOOST_PADS
        .get_or_init(|| {
            BOOST_PADS_COORDS
                .iter()
                .filter(|coords| (coords[2] - 70.0).abs() < f32::EPSILON)
                .cloned()
                .collect()
        })
        .to_vec()
}

pub fn boost_pad_distance_calculator() -> &'static BoostPadDistanceCalculator {
    static BOOST_PAD_DISTANCE_CALCULATOR: OnceLock<BoostPadDistanceCalculator> = OnceLock::new();
    BOOST_PAD_DISTANCE_CALCULATOR.get_or_init(BoostPadDistanceCalculator::new)
}

pub struct BoostPadDistanceCalculator {
    full_boost_pads: Array2<f32>,
    small_boost_pads: Array2<f32>,
    pub distance_buffer: f32,
}

impl BoostPadDistanceCalculator {
    pub fn new() -> Self {
        Self {
            full_boost_pads: arr2(&full_boost_pads()).slice(s![.., ..2]).to_owned(),
            small_boost_pads: arr2(&small_boost_pads()).slice(s![.., ..2]).to_owned(),
            distance_buffer: 50.0,
        }
    }

    pub fn calculate_boost_pad_collection_kind(
        &self,
        x: f32,
        y: f32,
    ) -> Result<BoostPickupKind, MinMaxError> {
        let full_boost_pad_minimum_distance =
            self.calculate_minimum_full_boost_pad_distance(x, y)?;
        let small_boost_pad_minimum_distance =
            self.calculate_minimum_small_boost_pad_distance(x, y)?;

        if full_boost_pad_minimum_distance < small_boost_pad_minimum_distance {
            self.warn_for_full_boost_pad_distance(full_boost_pad_minimum_distance);
            Ok(BoostPickupKind::Full)
        } else {
            self.warn_for_full_boost_pad_distance(small_boost_pad_minimum_distance);
            Ok(BoostPickupKind::Small)
        }
    }

    pub fn calculate_minimum_full_boost_pad_distance(
        &self,
        x: f32,
        y: f32,
    ) -> Result<f32, MinMaxError> {
        let position = arr2(&[[x, y]]);
        let full_boost_pad_distances = (&self.full_boost_pads - &position)
            .mapv(|a| a * a)
            .sum_axis(Axis(1))
            .mapv(f32::sqrt);
        Ok(*full_boost_pad_distances.min()?)
    }
    pub fn calculate_minimum_small_boost_pad_distance(
        &self,
        x: f32,
        y: f32,
    ) -> Result<f32, MinMaxError> {
        let position = arr2(&[[x, y]]);
        let small_boost_pad_distances = (&self.small_boost_pads - &position)
            .mapv(|a| a * a)
            .sum_axis(Axis(1))
            .mapv(f32::sqrt);
        Ok(*small_boost_pad_distances.min()?)
    }

    pub fn warn_for_full_boost_pad_distance(&self, full_boost_pad_minimum_distance: f32) {
        if full_boost_pad_minimum_distance > FULL_BOOST_PAD_RADIUS + self.distance_buffer {
            warn!(
                "Detected full boost pad collection from distance {} (typical pad radius: {})",
                full_boost_pad_minimum_distance, FULL_BOOST_PAD_RADIUS
            );
        }
    }
    pub fn warn_for_small_boost_pad_distance(&self, small_boost_pad_minimum_distance: f32) {
        if small_boost_pad_minimum_distance > SMALL_BOOST_PAD_RADIUS + self.distance_buffer {
            warn!(
                "Detected small boost pad collection from distance {} (typical pad radius: {})",
                small_boost_pad_minimum_distance, SMALL_BOOST_PAD_RADIUS
            );
        }
    }
}

impl Default for BoostPadDistanceCalculator {
    fn default() -> Self {
        BoostPadDistanceCalculator::new()
    }
}
