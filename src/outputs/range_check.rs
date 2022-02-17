use crate::outputs::DataFramesOutput;
use log::{debug, error};
use polars::series::Series;
use std::collections::HashMap;
use thiserror::Error;

/// Creates a map/set from an iterator, emulating a map/set literal syntax.
/// Taken from https://stackoverflow.com/a/27582993
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$($v,)*]))
    }};
}

static BALL_RADIUS: f32 = 92.75;
static BALL_MAX_SPEED: f32 = 6000.0;

static CAR_MAX_SPEED: f32 = 2300.0;

static PITCH_SIDE_WALL: f32 = 4096.0;
static PITCH_BACK_WALL: f32 = 5120.0;
static PITCH_FLOOR: f32 = 0.0;
static PITCH_CEILING: f32 = 2044.0;
static PITCH_GOAL_DEPTH: f32 = 880.0;

/// Checks the value ranges for the various parsed variables.
/// Verifies that the ranges are not only within know limits, but also span a reasonable amount of these limits.
/// This highlights possible parsing errors, likely caused by replay format differences.
/// For instance, old replay version may represent 100 as 1.00 for certain variables.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeChecker {
    pub ball: HashMap<String, Range>,
    pub player: HashMap<String, Range>,
}

impl RangeChecker {
    pub fn new() -> Self {
        let ball = collection! {
            "pos_x".to_string() => Range {
                min: -PITCH_SIDE_WALL + BALL_RADIUS - 30.0,
                max: PITCH_SIDE_WALL - BALL_RADIUS + 30.0,
                buffer: 60.0,
            },
            "pos_y".to_string() => Range {
                // Higher buffer due to ball being able to enter goal by a lag-dependent amount.
                min: -(PITCH_BACK_WALL + PITCH_GOAL_DEPTH - BALL_RADIUS) - 30.0,
                max: PITCH_BACK_WALL + PITCH_GOAL_DEPTH - BALL_RADIUS + 30.0,
                buffer: PITCH_GOAL_DEPTH + 60.0,
            },
            "pos_z".to_string() => Range {
                min: PITCH_FLOOR + BALL_RADIUS - 30.0,
                max: PITCH_CEILING - BALL_RADIUS + 30.0,
                buffer: 60.0,
            },
            "vel_x".to_string() => Range {
                min: -BALL_MAX_SPEED,
                max: BALL_MAX_SPEED,
                buffer: BALL_MAX_SPEED * 2.0 / 3.0,
            },
            "vel_y".to_string() => Range {
                min: -BALL_MAX_SPEED,
                max: BALL_MAX_SPEED,
                buffer: BALL_MAX_SPEED * 2.0 / 3.0,
            },
            "vel_z".to_string() => Range {
                // Large buffer as ball is not likely to be hit vertically with speed.
                min: -BALL_MAX_SPEED,
                max: BALL_MAX_SPEED,
                buffer: BALL_MAX_SPEED  * 5.0 / 6.0,
            },
            "quat_w".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
            "quat_x".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
            "quat_y".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
            "quat_z".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
        };
        let player = collection! {
            "pos_x".to_string() => Range {
                min: -PITCH_SIDE_WALL,
                max: PITCH_SIDE_WALL,
                buffer: PITCH_SIDE_WALL / 5.0,
            },
            "pos_y".to_string() => Range {
                min: -(PITCH_BACK_WALL + PITCH_GOAL_DEPTH),
                max: PITCH_BACK_WALL + PITCH_GOAL_DEPTH,
                buffer: PITCH_BACK_WALL / 5.0,
            },
            "pos_z".to_string() => Range {
                min: PITCH_FLOOR,
                max: PITCH_CEILING ,
                buffer: PITCH_CEILING / 2.0,
            },
            "vel_x".to_string() => Range {
                min: -CAR_MAX_SPEED,
                max: CAR_MAX_SPEED,
                buffer: CAR_MAX_SPEED / 2.0,
            },
            "vel_y".to_string() => Range {
                min: -CAR_MAX_SPEED,
                max: CAR_MAX_SPEED,
                buffer: CAR_MAX_SPEED / 2.0,
            },
            "vel_z".to_string() => Range {
                min: -CAR_MAX_SPEED,
                max: CAR_MAX_SPEED,
                buffer: CAR_MAX_SPEED / 2.0,
            },
            "quat_w".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
            "quat_x".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
            "quat_y".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
            "quat_z".to_string() => Range {
                min: -1.0 * 1.001,
                max: 1.0 * 1.001,
                buffer: 0.5,
            },
        };
        Self { ball, player }
    }

    pub fn check_ranges(&self, data_frames: &DataFramesOutput) -> Result<bool, RangeCheckerError> {
        let ball_df = &data_frames.ball;
        for (column_name, column_range) in self.ball.iter() {
            let series = ball_df
                .column(column_name)
                .map_err(|_| RangeCheckerError::MissingBallColumn(column_name.to_string()))?;
            if !column_range.check(series, &format!("ball {}", column_name))? {
                return Ok(false);
            }
        }

        for player_data_frame in data_frames.players.values() {
            for (column_name, column_range) in self.player.iter() {
                let series = player_data_frame
                    .column(column_name)
                    .map_err(|_| RangeCheckerError::MissingPlayerColumn(column_name.to_string()))?;
                if !column_range.check(series, &format!("player {}", column_name))? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

impl Default for RangeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    min: f32,
    max: f32,
    buffer: f32,
}

impl Range {
    pub fn check(&self, series: &Series, label: &str) -> Result<bool, RangeCheckerError> {
        let min = series
            .min::<f32>()
            .ok_or_else(|| RangeCheckerError::ArithmeticError(format!("{} min", label)))?;
        let max = series
            .max::<f32>()
            .ok_or_else(|| RangeCheckerError::ArithmeticError(format!("{} max", label)))?;
        debug!(
            "{}. found (min, max): ({}, {}), reference (min, max): ({}, {})",
            label, min, max, self.min, self.max,
        );
        if min < self.min || max > self.max {
            error!(
                "{} failed range check. found (min, max): ({}, {}), reference (min, max): ({}, {})",
                label, min, max, self.min, self.max,
            );
            return Ok(false);
        }

        if !is_close(min, self.min, self.buffer) {
            error!(
                "{} min failed range check. found: {}, reference: {}, buffer: {}",
                label, min, self.min, self.buffer
            );
            return Ok(false);
        }
        if !is_close(max, self.max, self.buffer) {
            error!(
                "{} max failed range check. found: {}, reference: {}, buffer: {}",
                label, max, self.max, self.buffer
            );
            return Ok(false);
        }
        Ok(true)
    }
}

pub fn is_close(value: f32, reference_value: f32, buffer: f32) -> bool {
    (value - reference_value).abs() < buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outputs::DataFramesOutput;
    use crate::CarballParser;
    use simplelog::*;
    use std::path::PathBuf;

    #[test]
    fn data_frames_value_ranges_expected() {
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            // TerminalMode::Mixed,
            TerminalMode::Stdout,
            ColorChoice::Auto,
        )])
        .unwrap();
        let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");
        // let file_path = PathBuf::from("assets\\replays\\rlcs-season-5-final.replay");
        // let file_path = PathBuf::from("assets\\replays\\soccar-lan.replay");
        let carball_parser = CarballParser::parse_file(file_path, false).expect("failed to parse");
        dbg!(&carball_parser.frame_parser.replay_version);
        let data_frames = DataFramesOutput::generate_from(&carball_parser.frame_parser).unwrap();

        let range_checker = RangeChecker::new();
        assert!(range_checker.check_ranges(&data_frames).unwrap());
        // assert_eq!(2 + 2, 4);
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RangeCheckerError {
    #[error("missing ball column: {0}")]
    MissingBallColumn(String),
    #[error("missing player column: {0}")]
    MissingPlayerColumn(String),
    #[error("calculation failed: {0}")]
    ArithmeticError(String),
}
