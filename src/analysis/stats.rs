use crate::actor_handlers::WrappedUniqueId;
use crate::analysis::GameplayPeriod;
use crate::outputs::{DataFramesOutput, MetadataOutput, Player};
use log::warn;
use polars::error::PolarsError;
use polars::prelude::{BooleanChunked, ChunkAgg, ChunkApply, ChunkCompare, ChunkFilter, DataFrame};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

static BOOST_PER_SECOND: f32 = 85.0 / 2.55;
static PITCH_Y_THIRD_THRESHOLD: f32 = 10240.0 / 3.0 / 2.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stats {
    pub players: HashMap<WrappedUniqueId, PlayerStats>,
}

impl Stats {
    pub fn generate_from(
        metadata: &MetadataOutput,
        data_frames: &DataFramesOutput,
        gameplay_periods: &[GameplayPeriod],
    ) -> Result<Self, StatsGenerationError> {
        let mut gameplay_frames_set = HashSet::new();
        for gameplay_period in gameplay_periods.iter() {
            for frame_number in gameplay_period.start_frame..gameplay_period.end_frame {
                gameplay_frames_set.insert(frame_number);
            }
        }
        let mut gameplay_frames_boolean_vec = vec![];
        for frame_number in 0..metadata.game.num_frames {
            gameplay_frames_boolean_vec.push(gameplay_frames_set.contains(&frame_number));
        }
        let gameplay_frames_boolean_mask: BooleanChunked =
            gameplay_frames_boolean_vec.into_iter().collect();

        let game_df = data_frames
            .game
            .filter(&gameplay_frames_boolean_mask)
            .unwrap();

        let mut players_stats: HashMap<WrappedUniqueId, PlayerStats> = HashMap::new();
        for player in metadata.players.iter() {
            if let Some(player_df) = data_frames.players.get(&player.unique_id) {
                match PlayerStats::from(
                    player,
                    &player_df.filter(&gameplay_frames_boolean_mask).unwrap(),
                    &game_df,
                ) {
                    Ok(player_stats) => {
                        players_stats.insert(player.unique_id.clone(), player_stats);
                    }
                    Err(e) => {
                        warn!("Failed to generate stats for {} {:?}", player.name, e);
                    }
                }
            } else {
                warn!(
                    "Not generating player stats for {} as missing data frame (unique id: {})",
                    player.name,
                    player.unique_id.to_string()
                )
            }
        }
        Ok(Self {
            players: players_stats,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PlayerStats {
    // Boost
    pub big_pads_collected: u32,
    pub small_pads_collected: u32,
    // pub stolen_boosts: u32,
    pub boost_used: f32,
    pub time_full_boost: f32,
    pub time_high_boost: f32,
    pub time_low_boost: f32,
    pub time_no_boost: f32,
    pub average_boost_level: f32,

    // Movement
    pub average_speed: f32,
    pub time_at_supersonic: f32,
    pub time_at_boost_speed: f32,
    pub time_at_slow_speed: f32,

    // Positioning
    // pub time_high_in_air: f32,
    // pub time_in_air: f32,
    pub time_on_ground: f32,
    pub time_near_ground: f32,
    pub time_in_attacking_half: f32,
    pub time_in_defending_half: f32,
    pub time_in_attacking_third: f32,
    pub time_in_neutral_third: f32,
    pub time_in_defending_third: f32,
}

impl PlayerStats {
    pub fn from(
        player: &Player,
        player_df: &DataFrame,
        game_df: &DataFrame,
    ) -> Result<Self, PlayerStatsError> {
        let boost_pickup = player_df.column("boost_pickup")?.u8()?;

        let boost_amount = player_df.column("boost_amount")?.f32()?;
        let game_delta = game_df.column("delta")?.f32()?;
        let total_game_delta = game_delta.sum().unwrap();

        let vel_x = player_df.column("vel_x")?.f32()?;
        let vel_y = player_df.column("vel_y")?.f32()?;
        let vel_z = player_df.column("vel_z")?.f32()?;
        let speed = (vel_x.apply_values(|v| v * v) + vel_y.apply_values(|v| v * v) + vel_z.apply_values(|v| v * v))
            .apply_values(f32::sqrt);

        let pos_y = player_df.column("pos_y")?.f32()?;
        let pos_z = player_df.column("pos_z")?.f32()?;

        let time_in_blue_half = game_delta.filter(&pos_y.lt(0.0_f32))?.sum().unwrap_or(0.0);
        let time_in_orange_half = game_delta.filter(&pos_y.gt(0.0_f32))?.sum().unwrap_or(0.0);
        let time_in_blue_third = game_delta
            .filter(&pos_y.lt(-PITCH_Y_THIRD_THRESHOLD))?
            .sum()
            .unwrap_or(0.0);
        let time_in_neutral_third = game_delta
            .filter(&pos_y.apply_values(f32::abs).lt(PITCH_Y_THIRD_THRESHOLD))?
            .sum()
            .unwrap_or(0.0);
        let time_in_orange_third = game_delta
            .filter(&pos_y.gt(PITCH_Y_THIRD_THRESHOLD))?
            .sum()
            .unwrap_or(0.0);

        let time_in_attacking_half;
        let time_in_defending_half;
        let time_in_attacking_third;
        let time_in_defending_third;

        let is_orange = player.is_orange.ok_or(PlayerStatsError::NoTeamColor)?;
        match is_orange {
            true => {
                time_in_attacking_half = time_in_blue_half;
                time_in_defending_half = time_in_orange_half;
                time_in_attacking_third = time_in_blue_third;
                time_in_defending_third = time_in_orange_third;
            }
            false => {
                time_in_attacking_half = time_in_orange_half;
                time_in_defending_half = time_in_blue_half;
                time_in_attacking_third = time_in_orange_third;
                time_in_defending_third = time_in_blue_third;
            }
        }

        Ok(Self {
            big_pads_collected: boost_pickup.equal(2).sum().unwrap_or(0_u32),
            small_pads_collected: boost_pickup.equal(1).sum().unwrap_or(0),
            boost_used: game_delta
                .filter(&player_df.column("boost_is_active")?.u8()?.equal(1_i32))?
                .sum()
                .unwrap_or(0.0)
                * BOOST_PER_SECOND,
            time_full_boost: game_delta
                .filter(&boost_amount.gt_eq(95.0_f32))?
                .sum()
                .unwrap_or(0.0),
            time_high_boost: game_delta
                .filter(&boost_amount.gt_eq(75.0_f32))?
                .sum()
                .unwrap_or(0.0),
            time_low_boost: game_delta
                .filter(&boost_amount.lt_eq(25.0_f32))?
                .sum()
                .unwrap_or(0.0),
            time_no_boost: game_delta
                .filter(&boost_amount.lt_eq(5.0_f32))?
                .sum()
                .unwrap_or(0.0),
            average_boost_level: (game_delta * boost_amount).sum().unwrap_or(0.0)
                / total_game_delta,

            average_speed: (game_delta * &speed).sum().unwrap_or(0 as f32) / total_game_delta,
            time_at_supersonic: game_delta
                .filter(&speed.gt(2200.0_f32))?
                .sum()
                .unwrap_or(0.0),
            time_at_boost_speed: game_delta
                .filter(&speed.gt(1450.0_f32))?
                .sum()
                .unwrap_or(0.0),
            time_at_slow_speed: game_delta
                .filter(&speed.lt(700.0_f32))?
                .sum()
                .unwrap_or(0.0),

            time_on_ground: game_delta.filter(&pos_z.lt(20.0_f32))?.sum().unwrap_or(0.0),
            time_near_ground: game_delta
                .filter(&pos_z.lt(150.0_f32))?
                .sum()
                .unwrap_or(0.0),
            time_in_attacking_half,
            time_in_defending_half,
            time_in_attacking_third,
            time_in_neutral_third,
            time_in_defending_third,
        })
    }
}

#[derive(Error, Debug)]
pub enum StatsGenerationError {
    // TODO: Reintroduce stats generation errors.
// #[error("Player stats generation Polars error: {0}")]
// PlayerStatsError(PolarsError),
}

#[derive(Error, Debug)]
pub enum PlayerStatsError {
    #[error("Player stats generation Polars error: {0}")]
    PolarsError(PolarsError),
    #[error("Player's is_orange attribute is null")]
    NoTeamColor,
}

impl From<PolarsError> for PlayerStatsError {
    fn from(error: PolarsError) -> Self {
        Self::PolarsError(error)
    }
}
