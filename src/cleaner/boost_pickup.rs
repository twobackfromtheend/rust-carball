use crate::actor_handlers::TimeSeriesCarData;
use crate::cleaner::boost_pad::BOOST_PAD_DISTANCE_CALCULATOR;
use crate::cleaner::SMALL_BOOST_PAD_RADIUS;
use crate::cleaner::SPAWN_DISTANCE_CALCULATOR;
use log::warn;
use ndarray_stats::errors::MinMaxError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoostPickupKind {
    Full,
    Small,
}

impl BoostPickupKind {
    pub fn detect_boost_pickup_kind(
        last_boost_amount: f32,
        new_boost_amount: f32,
        car_data: Option<&TimeSeriesCarData>,
        frame_number: usize,
        player_name: &str,
    ) -> Result<Option<BoostPickupKind>, BoostPickupKindCalculationError> {
        let boost_increase = new_boost_amount - last_boost_amount;

        let nearly_full_after_pickup = (100.0 - new_boost_amount) < 3.0;
        let boost_increase_is_insignificant = boost_increase < 2.0;
        let boost_increase_is_small = boost_increase < 14.0;
        let boost_increase_is_nearly_small_amount =
            10.0 < boost_increase && boost_increase_is_small;
        let boost_amount_near_default = (new_boost_amount - (85.0 / 2.55)).abs() < 0.1;

        if boost_increase_is_small {
            // Small increase. Likely a small boost pad pickup.
            // One of:
            //     a) Nearly full after pickup. Must then check if it's a full boost pad pickup.
            //     b) Close to small boost pad amount. Must check if it's a spawn or respawn.
            //     c) Very small increase. Ignore due to imprecise boost usage calculation.
            //     d) Other: Not full boost, not close to small boost pad amount, but significant.
            //         Could be respawn.
            //         Treat as small boost pad pickup, with delayed boost amount update.

            if nearly_full_after_pickup {
                // Cannot differentiate between big and small boost pickup from increase.
                if let Some(_car_data) = car_data {
                    return Ok(Some(
                        BOOST_PAD_DISTANCE_CALCULATOR
                            .calculate_boost_pad_collection_kind(
                                _car_data.pos_x.ok_or(
                                    BoostPickupKindCalculationError::MissingCarPositionData,
                                )?,
                                _car_data.pos_y.ok_or(
                                    BoostPickupKindCalculationError::MissingCarPositionData,
                                )?,
                            )
                            .map_err(
                                BoostPickupKindCalculationError::FailedToCalculateBoostDistanceMin,
                            )?,
                    ));
                }
                warn!(
                    "Cannot infer boost pickup kind as car data is missing. (Frame {}; {:?}) (boost_increase_is_small, nearly_full_after_pickup)",
                    frame_number, player_name
                );
                return Ok(None);
            } else if boost_increase_is_nearly_small_amount {
                if boost_amount_near_default {
                    if let Some(_car_data) = car_data {
                        let x = _car_data
                            .pos_x
                            .ok_or(BoostPickupKindCalculationError::MissingCarPositionData)?;
                        let y = _car_data
                            .pos_y
                            .ok_or(BoostPickupKindCalculationError::MissingCarPositionData)?;
                        let small_boost_pad_minimum_distance = BOOST_PAD_DISTANCE_CALCULATOR
                            .calculate_minimum_small_boost_pad_distance(x, y)
                            .map_err(
                                BoostPickupKindCalculationError::FailedToCalculateBoostDistanceMin,
                            )?;
                        let is_near_small_boost_pad = small_boost_pad_minimum_distance
                            < SMALL_BOOST_PAD_RADIUS
                                + BOOST_PAD_DISTANCE_CALCULATOR.distance_buffer;
                        let is_near_spawn_or_respawn = SPAWN_DISTANCE_CALCULATOR
                            .check_if_near_spawn(x, y)
                            .map_err(
                                BoostPickupKindCalculationError::FailedToCalculateSpawnDistanceMin,
                            )?;
                        if is_near_small_boost_pad && !is_near_spawn_or_respawn {
                            return Ok(Some(BoostPickupKind::Small));
                        } else if !is_near_small_boost_pad && is_near_spawn_or_respawn {
                            return Ok(None);
                        } else if is_near_small_boost_pad && is_near_spawn_or_respawn {
                            warn!(
                                "Cannot tell if respawn or boost collect. (Frame {}; {:?})",
                                frame_number, player_name
                            );
                        } else if !is_near_small_boost_pad && !is_near_spawn_or_respawn {
                            return Ok(None);
                        } else {
                            panic!("Code should not reach here as all boolean combinations should be covered.");
                        }
                    } else {
                        warn!(
                            "Cannot infer boost pickup kind as car data is missing. (Frame {}; {:?}) (boost_increase_is_small, boost_increase_is_nearly_small_amount)",
                            frame_number, player_name
                        );
                    }
                }
                return Ok(Some(BoostPickupKind::Small));
            } else if boost_increase_is_insignificant {
                // Boost increase is likely negligible and is due to boost usage overestimation (i.e. boost amount increases slightly when updated true value comes through).
                return Ok(None);
            } else if boost_amount_near_default {
                if let Some(_car_data) = car_data {
                    let x = _car_data
                        .pos_x
                        .ok_or(BoostPickupKindCalculationError::MissingCarPositionData)?;
                    let y = _car_data
                        .pos_y
                        .ok_or(BoostPickupKindCalculationError::MissingCarPositionData)?;
                    if SPAWN_DISTANCE_CALCULATOR
                        .check_if_near_spawn(x, y)
                        .map_err(
                            BoostPickupKindCalculationError::FailedToCalculateSpawnDistanceMin,
                        )?
                    {
                        // Spawn
                        return Ok(None);
                    } else {
                        warn!(
                            "Boost pickup is small but significant ({:.1} to {:.1}). Assuming small boost pad was picked up but update was delayed in replay. (Frame {}; {:?}) (boost_amount_near_default)",
                            last_boost_amount, new_boost_amount, frame_number, player_name,
                        );
                        return Ok(Some(BoostPickupKind::Small));
                    };
                } else {
                    // Spawn (as lacking car data and boost amount near default)
                    return Ok(None);
                };
            } else {
                warn!(
                    "Boost pickup is small but significant ({:.1} to {:.1}). Assuming small boost pad was picked up but update was delayed in replay. (Frame {}; {:?})",
                    last_boost_amount, new_boost_amount, frame_number, player_name,
                );
                return Ok(Some(BoostPickupKind::Small));
            }
        } else if nearly_full_after_pickup {
            return Ok(Some(BoostPickupKind::Full));
        } else if !boost_amount_near_default {
            warn!(
                "Boost pickup is large, but is not near full after ({:.1} to {:.1}). Cannot infer what happened. (Frame {}; {:?})",
                last_boost_amount, new_boost_amount, frame_number, player_name,
            );
        }
        Ok(None)
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum BoostPickupKindCalculationError {
    #[error("missing car position data")]
    MissingCarPositionData,
    #[error("failed to calculate minimum distance to boost pad")]
    FailedToCalculateBoostDistanceMin(MinMaxError),
    #[error("failed to calculate minimum distance to spawn location")]
    FailedToCalculateSpawnDistanceMin(MinMaxError),
}
