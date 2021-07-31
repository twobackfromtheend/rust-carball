use crate::actor_handlers::WrappedUniqueId;
use crate::outputs::{DataFramesOutput, MetadataOutput, Player};
use log::{info, warn};
use polars::error::PolarsError;
use polars::prelude::{
    AnyValue, BooleanChunked, ChunkAgg, ChunkApply, ChunkCast, ChunkCompare, ChunkFilter,
    DataFrame, Series, UInt32Type,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
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
    ) -> Result<Self, StatsGenerationError> {
        let gameplay_periods = GameplayPeriod::get_periods(metadata, data_frames);

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
                let player_stats = PlayerStats::from(
                    player,
                    &player_df.filter(&gameplay_frames_boolean_mask).unwrap(),
                    &game_df,
                )
                .map_err(StatsGenerationError::PlayerStatsError)?;
                players_stats.insert(player.unique_id.clone(), player_stats);
                // info!("{} {:?}", player.name, player_stats);
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
    ) -> Result<Self, PolarsError> {
        let boost_pickup = player_df
            .column("boost_pickup")?
            .u8()?
            .cast::<UInt32Type>()?;

        let boost_amount = player_df.column("boost_amount")?.f32()?;
        let game_delta = game_df.column("delta")?.f32()?;
        let total_game_delta = game_delta.sum().unwrap();

        let vel_x = player_df.column("vel_x")?.f32()?;
        let vel_y = player_df.column("vel_y")?.f32()?;
        let vel_z = player_df.column("vel_z")?.f32()?;
        let speed = (vel_x.apply(|v| v * v) + vel_y.apply(|v| v * v) + vel_z.apply(|v| v * v))
            .apply(f32::sqrt);

        let pos_y = player_df.column("pos_y")?.f32()?;
        let pos_z = player_df.column("pos_z")?.f32()?;

        let time_in_blue_half = game_delta.filter(&pos_y.lt(0.0))?.sum().unwrap();
        let time_in_orange_half = game_delta.filter(&pos_y.gt(0.0))?.sum().unwrap();
        let time_in_blue_third = game_delta
            .filter(&pos_y.lt(-PITCH_Y_THIRD_THRESHOLD))?
            .sum()
            .unwrap();
        let time_in_neutral_third = game_delta
            .filter(&pos_y.apply(f32::abs).lt(PITCH_Y_THIRD_THRESHOLD))?
            .sum()
            .unwrap();
        let time_in_orange_third = game_delta
            .filter(&pos_y.gt(PITCH_Y_THIRD_THRESHOLD))?
            .sum()
            .unwrap();

        let time_in_attacking_half;
        let time_in_defending_half;
        let time_in_attacking_third;
        let time_in_defending_third;
        if player.is_orange == None {
            dbg!(player);
        }
        match player.is_orange.unwrap() {
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
            big_pads_collected: boost_pickup.eq(2).sum().unwrap(),
            small_pads_collected: boost_pickup.eq(1).sum().unwrap(),
            boost_used: game_delta
                .filter(player_df.column("boost_is_active")?.bool()?)?
                .sum()
                .unwrap()
                * BOOST_PER_SECOND,
            time_full_boost: game_delta.filter(&boost_amount.gt_eq(95.0))?.sum().unwrap(),
            time_high_boost: game_delta.filter(&boost_amount.gt_eq(75.0))?.sum().unwrap(),
            time_low_boost: game_delta.filter(&boost_amount.lt_eq(25.0))?.sum().unwrap(),
            time_no_boost: game_delta.filter(&boost_amount.lt_eq(5.0))?.sum().unwrap(),
            average_boost_level: (game_delta * boost_amount).sum().unwrap() / total_game_delta,

            average_speed: (game_delta * &speed).sum().unwrap() / total_game_delta,
            time_at_supersonic: game_delta.filter(&speed.gt(2200.0))?.sum().unwrap(),
            time_at_boost_speed: game_delta.filter(&speed.gt(1450.0))?.sum().unwrap(),
            time_at_slow_speed: game_delta.filter(&speed.lt(700.0))?.sum().unwrap(),

            time_on_ground: game_delta.filter(&pos_z.lt(20.0))?.sum().unwrap(),
            time_near_ground: game_delta.filter(&pos_z.lt(150.0))?.sum().unwrap(),
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
    #[error("Player stats generation Polars error: {0}")]
    PlayerStatsError(PolarsError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayPeriod {
    pub start_frame: i32,
    pub end_frame: i32,
    pub first_hit_frame: i32,
    pub goal_frame: Option<i32>,
}

impl GameplayPeriod {
    pub fn get_periods(metadata: &MetadataOutput, data_frames: &DataFramesOutput) -> Vec<Self> {
        let replicated_game_state_time_remaining = data_frames
            .game
            .column("replicated_game_state_time_remaining")
            .unwrap();
        let hit_team_num = data_frames.ball.column("hit_team_num").unwrap();
        let game_frames: usize = metadata.game.num_frames.try_into().unwrap();

        let mut gameplay_periods = vec![];

        let mut start_search_at: usize = 0;
        for goal in metadata.game.goals.iter() {
            let goal_frame: usize = goal.frame.try_into().unwrap();
            let start_frame = GameplayPeriod::find_start_frame(
                replicated_game_state_time_remaining,
                start_search_at,
                goal_frame,
            );

            let end_frame =
                GameplayPeriod::find_end_frame(hit_team_num, goal_frame, game_frames - 1);

            let first_hit_frame =
                GameplayPeriod::find_first_hit_frame(hit_team_num, start_frame, goal_frame);

            info!(
                "gameplay period for goal: {} to {} (first hit at {}, goal at {})",
                start_frame, end_frame, first_hit_frame, goal_frame
            );

            gameplay_periods.push(GameplayPeriod {
                start_frame: start_frame.try_into().unwrap(),
                end_frame: end_frame.try_into().unwrap(),
                first_hit_frame: first_hit_frame.try_into().unwrap(),
                goal_frame: Some(goal_frame.try_into().unwrap()),
            });

            // Set start_search_at for next gameplay period.
            start_search_at = end_frame + 1;
        }

        if start_search_at < game_frames - 1 {
            // Last gameplay period without a goal.
            let start_frame = GameplayPeriod::find_start_frame(
                replicated_game_state_time_remaining,
                start_search_at,
                game_frames - 1,
            );
            let end_frame = game_frames - 1;
            let first_hit_frame =
                GameplayPeriod::find_first_hit_frame(hit_team_num, start_frame, end_frame);

            info!(
                "gameplay period (final, no goal): {} to {} (first hit at {})",
                start_frame, end_frame, first_hit_frame
            );

            gameplay_periods.push(GameplayPeriod {
                start_frame: start_frame.try_into().unwrap(),
                end_frame: end_frame.try_into().unwrap(),
                first_hit_frame: first_hit_frame.try_into().unwrap(),
                goal_frame: None,
            });
        }
        gameplay_periods
    }

    /// Find start frame with replicated_game_state_time_remaining = 0 (meaning countdown has elapsed).
    fn find_start_frame(
        replicated_game_state_time_remaining: &Series,
        start_search_at: usize,
        end_search_at: usize,
    ) -> usize {
        let mut search_start_frame = start_search_at;
        loop {
            if let AnyValue::Int32(0) = replicated_game_state_time_remaining.get(search_start_frame)
            {
                break;
            }
            search_start_frame += 1;
            if search_start_frame >= end_search_at {
                panic!("Could not find start frame for gameplay period.");
            }
        }
        search_start_frame
    }

    /// Find end frame as the frame before hit_team_num = not set.
    /// Unlike other find_X_frame functions, end_search_at is not treated as an error.
    fn find_end_frame(
        hit_team_num: &Series,
        start_search_at: usize,
        end_search_at: usize,
    ) -> usize {
        let mut search_end_frame: usize = start_search_at;
        loop {
            if let AnyValue::Null = hit_team_num.get(search_end_frame) {
                // Set to last frame where hit_team_num is set.
                search_end_frame -= 1;
                break;
            }
            search_end_frame += 1;
            if search_end_frame >= end_search_at {
                break;
            } else if search_end_frame >= start_search_at + 500 {
                panic!("Could not find end frame for gameplay period.");
            }
        }
        search_end_frame
    }

    /// Find first hit frame as frame where hit_team_num is set.
    fn find_first_hit_frame(
        hit_team_num: &Series,
        start_search_at: usize,
        end_search_at: usize,
    ) -> usize {
        let mut search_hit_frame: usize = start_search_at;
        loop {
            if let AnyValue::UInt8(_) = hit_team_num.get(search_hit_frame) {
                break;
            }
            search_hit_frame += 1;
            if search_hit_frame >= end_search_at {
                panic!("Could not find first hit frame for gameplay period.");
            }
        }
        search_hit_frame
    }
}
