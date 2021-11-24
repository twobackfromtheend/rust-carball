use crate::outputs::{DataFramesOutput, MetadataOutput};
use log::info;
use polars::prelude::{AnyValue, Series};
use serde::Serialize;
use std::convert::TryInto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

        // Buffer of a couple of frames because some replays have shenanigans happening after (ball teleporting and HitTeamNum not set then set to other team).
        if start_search_at < game_frames - 20 {
            // Last gameplay period without a goal.
            let start_frame = GameplayPeriod::find_start_frame(
                replicated_game_state_time_remaining,
                start_search_at,
                game_frames - 1,
            );
            let end_frame = game_frames - 1;
            let first_hit_frame =
                GameplayPeriod::find_first_hit_frame(hit_team_num, start_frame, end_frame);

            // info!(
            //     "gameplay period (final, no goal): {} to {} (first hit at {})",
            //     start_frame, end_frame, first_hit_frame
            // );

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
                println!("{}, {}", start_search_at, end_search_at);
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
