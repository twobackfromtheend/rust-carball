use crate::actor_handlers::{TimeSeriesBallData, TimeSeriesCarData, WrappedUniqueId};
use crate::analysis::{predict_ball_bounce, BallPredictionError};
use crate::frame_parser::FrameParser;
use crate::outputs::MetadataOutput;
use log::{error, warn};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use thiserror::Error;

static MAX_HIT_CAR_DISTANCE: f32 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IsHitConclusion {
    Hit,
    PossibleHit,
    NotHit,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hit {
    pub frame_number: usize,
    pub player_unique_id: WrappedUniqueId,
    pub player_distance: f32,
    pub _debug_info: HitDebugInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HitDebugInfo {
    pub hit_team_num_changed: bool,
    pub ang_vel_changed: bool,
    pub predicted_bounce: bool,
    pub speed_increased: bool,
}

impl Hit {
    pub fn find_hits(
        frame_parser: &FrameParser,
        metadata: &MetadataOutput,
    ) -> Result<Vec<Hit>, HitDetectionError> {
        let time_series_replay_data = frame_parser.time_series_replay_data.borrow();
        let time_series_ball_data = frame_parser.time_series_ball_data.borrow();
        let players_time_series_car_data = frame_parser.players_time_series_car_data.borrow();

        let mut blue_players_wrapped_unique_id = vec![];
        let mut orange_players_wrapped_unique_id = vec![];
        for player in metadata.players.iter() {
            if let Some(player_is_orange) = player.is_orange {
                match player_is_orange {
                    true => orange_players_wrapped_unique_id.push(player.unique_id.clone()),
                    false => blue_players_wrapped_unique_id.push(player.unique_id.clone()),
                }
            }
        }

        let mut hits = vec![];
        let mut previous_frame_ball_data: Option<&TimeSeriesBallData> = None;
        for frame_number in 0..(frame_parser.frame_count - 1) {
            if let Some(ball_data) = time_series_ball_data.get(&frame_number) {
                match previous_frame_ball_data {
                    None => {
                        previous_frame_ball_data = Some(ball_data);
                        continue;
                    }
                    Some(previous_frame_ball_data_value) => {
                        if let Some(hit_team_num) = ball_data.hit_team_num {
                            if let Some((ang_vel_x, ang_vel_y, ang_vel_z)) =
                                unwrap_ang_vel(ball_data)
                            {
                                // Detect hits
                                let mut hit_team_num_changed = false;
                                let mut ang_vel_changed = false;
                                let mut predicted_bounce = false;
                                let mut speed_increased = false;

                                if hit_team_num
                                    != previous_frame_ball_data_value.hit_team_num.unwrap_or(255)
                                {
                                    hit_team_num_changed = true;
                                }
                                if (ang_vel_x
                                    - previous_frame_ball_data_value.ang_vel_x.unwrap_or(0.0))
                                .abs()
                                    > f32::EPSILON
                                    || (ang_vel_y
                                        - previous_frame_ball_data_value.ang_vel_y.unwrap_or(0.0))
                                    .abs()
                                        > f32::EPSILON
                                    || (ang_vel_z
                                        - previous_frame_ball_data_value.ang_vel_z.unwrap_or(0.0))
                                    .abs()
                                        > f32::EPSILON
                                {
                                    ang_vel_changed = true;
                                }
                                let delta = time_series_replay_data
                                    .get(&frame_number)
                                    .ok_or(HitDetectionError::MissingDelta(frame_number))?
                                    .delta;
                                {
                                    if predict_ball_bounce(previous_frame_ball_data_value, delta)
                                        .map_err(HitDetectionError::BallPredictionError)?
                                    {
                                        predicted_bounce = true;
                                    };
                                }

                                if let Some(_previous_frame_ball_data) = previous_frame_ball_data {
                                    let previous_ball_speed =
                                        get_ball_speed(_previous_frame_ball_data);
                                    let current_ball_speed = get_ball_speed(ball_data);
                                    if let Some(_current_ball_speed) = current_ball_speed {
                                        if let Some(_previous_ball_speed) = previous_ball_speed {
                                            if _current_ball_speed
                                                > _previous_ball_speed + 650.0 * delta
                                            {
                                                speed_increased = true;
                                            }
                                        }
                                    }
                                }

                                {
                                    // info!(
                                    //     "\t{} {} {} {}",
                                    //     hit_team_num_changed,
                                    //     ang_vel_changed,
                                    //     predicted_bounce,
                                    //     speed_increased
                                    // );
                                    // if hit_team_num_changed => hit
                                    // if ang_vel_changed => hit or bounce
                                    // if speed_increased => hit or lag correction?
                                    let mut is_hit: IsHitConclusion = IsHitConclusion::NotHit;
                                    if hit_team_num_changed {
                                        is_hit = IsHitConclusion::Hit;
                                    } else if ang_vel_changed {
                                        if speed_increased {
                                            is_hit = IsHitConclusion::Hit;
                                        } else {
                                            // No definite conclusion here
                                            // hit_team_num did not change, speed did not increase, but ang_vel changed
                                            if !predicted_bounce {
                                                // Look at nearest car distance.
                                                is_hit = IsHitConclusion::PossibleHit;
                                            }
                                        }
                                    };

                                    if is_hit == IsHitConclusion::Hit
                                        || is_hit == IsHitConclusion::PossibleHit
                                    {
                                        // Filter potential hit players by hit_team_num
                                        let empty_vec = vec![];
                                        let potential_hit_players = match hit_team_num {
                                            0 => &blue_players_wrapped_unique_id,
                                            1 => &orange_players_wrapped_unique_id,
                                            _ => &empty_vec,
                                        };

                                        let potential_hit_player_previous_frame_datas: HashMap<
                                            WrappedUniqueId,
                                            Option<&TimeSeriesCarData>,
                                        > = potential_hit_players
                                            .iter()
                                            .map(|wrapped_unique_id| {
                                                (
                                                    wrapped_unique_id.clone(),
                                                    players_time_series_car_data
                                                        .get(wrapped_unique_id)
                                                        .and_then(|time_series_car_data| {
                                                            time_series_car_data
                                                                .get(&(frame_number - 1))
                                                        }),
                                                )
                                            })
                                            .collect();

                                        let player_distances = get_player_distances(
                                            previous_frame_ball_data_value,
                                            potential_hit_player_previous_frame_datas,
                                        );
                                        let nearest_player_and_distance =
                                            get_nearest_player(player_distances);
                                        if is_hit == IsHitConclusion::Hit {
                                            // Immediately find nearest player
                                            if let Some((nearest_player, nearest_distance)) =
                                                nearest_player_and_distance
                                            {
                                                if nearest_distance > MAX_HIT_CAR_DISTANCE {
                                                    warn!(
                                                        "Found hit on frame {} where nearest player ({}) is far from ball: {} uu. Hit is ignored.", 
                                                        frame_number,
                                                        nearest_player.to_string(), 
                                                        nearest_distance,
                                                    );
                                                } else {
                                                    hits.push(Hit {
                                                        frame_number,
                                                        player_unique_id: nearest_player.clone(),
                                                        player_distance: nearest_distance,
                                                        _debug_info: HitDebugInfo {
                                                            hit_team_num_changed,
                                                            ang_vel_changed,
                                                            predicted_bounce,
                                                            speed_increased,
                                                        },
                                                    });
                                                }
                                            } else {
                                                error!("Failed to find nearest player on frame {} for hit.", frame_number);
                                            }
                                        } else if is_hit == IsHitConclusion::PossibleHit {
                                            if let Some((nearest_player, nearest_distance)) =
                                                nearest_player_and_distance
                                            {
                                                if nearest_distance <= MAX_HIT_CAR_DISTANCE {
                                                    hits.push(Hit {
                                                        frame_number,
                                                        player_unique_id: nearest_player,
                                                        player_distance: nearest_distance,
                                                        _debug_info: HitDebugInfo {
                                                            hit_team_num_changed,
                                                            ang_vel_changed,
                                                            predicted_bounce,
                                                            speed_increased,
                                                        },
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        previous_frame_ball_data = Some(ball_data);
                    }
                }
            }
        }
        Ok(hits)
    }
}

fn get_player_distances(
    ball_data: &TimeSeriesBallData,
    player_datas: HashMap<WrappedUniqueId, Option<&TimeSeriesCarData>>,
) -> HashMap<WrappedUniqueId, f32> {
    let mut player_distances = HashMap::with_capacity(player_datas.len());
    for (wrapped_unique_id, player_data) in player_datas.iter() {
        if let Some(_player_data) = player_data {
            if let Some(distance) = get_player_distance(ball_data, _player_data) {
                player_distances.insert(wrapped_unique_id.clone(), distance);
            };
        }
    }
    player_distances
}

fn get_player_distance(
    ball_data: &TimeSeriesBallData,
    player_data: &TimeSeriesCarData,
) -> Option<f32> {
    let pos_x_displacement = ball_data.pos_x? - player_data.pos_x?;
    let pos_y_displacement = ball_data.pos_y? - player_data.pos_y?;
    let pos_z_displacement = ball_data.pos_z? - player_data.pos_z?;
    Some(
        (pos_x_displacement * pos_x_displacement
            + pos_y_displacement * pos_y_displacement
            + pos_z_displacement * pos_z_displacement)
            .sqrt(),
    )
}

fn get_nearest_player(
    player_distances: HashMap<WrappedUniqueId, f32>,
) -> Option<(WrappedUniqueId, f32)> {
    player_distances
        .iter()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Greater))
        .map(|(k, v)| (k.clone(), *v))
}

fn get_ball_speed(ball_data: &TimeSeriesBallData) -> Option<f32> {
    Some(
        (ball_data.vel_x? * ball_data.vel_x?
            + ball_data.vel_y? * ball_data.vel_y?
            + ball_data.vel_z? * ball_data.vel_z?)
            .sqrt(),
    )
}

fn unwrap_ang_vel(ball_data: &TimeSeriesBallData) -> Option<(f32, f32, f32)> {
    Some((
        ball_data.ang_vel_x?,
        ball_data.ang_vel_y?,
        ball_data.ang_vel_z?,
    ))
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitDetectionError {
    #[error("ball prediction failed: {0}")]
    BallPredictionError(BallPredictionError),
    #[error("missing delta from parsed replay on frame {0}")]
    MissingDelta(usize),
}
