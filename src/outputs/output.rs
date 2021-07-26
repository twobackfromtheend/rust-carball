use crate::actor_handlers::{
    TimeSeriesBallData, TimeSeriesBoostData, TimeSeriesCarData, TimeSeriesGameEventData,
    TimeSeriesPlayerData,
};
use crate::cleaner::BoostPickupKind;
use crate::frame_parser::{FrameParser, TimeSeriesReplayData};
use crate::outputs::{Demo, Game, Player, Team};
use boxcars::{Attribute, Replay};
use log::error;
use polars::error::PolarsError;
use polars::prelude::{
    BooleanChunked, DataFrame, Float32Chunked, Int32Chunked, IntoSeries, NewChunkedArray,
    UInt8Chunked,
};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataOutput {
    pub game: Game,
    pub teams: Vec<Team>,
    pub players: Vec<Player>,
    pub demos: Vec<Demo>,
}

impl MetadataOutput {
    pub fn generate_from(replay: &Replay, frame_parser: &FrameParser) -> Self {
        Self {
            game: Game::from(replay),
            teams: Team::from_frame_parser(frame_parser),
            players: Player::from_frame_parser(frame_parser),
            demos: Demo::from_frame_parser(frame_parser),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataFramesOutput {
    pub game: DataFrame,
    pub ball: DataFrame,
    pub players: HashMap<i32, DataFrame>,
}

impl DataFramesOutput {
    pub fn generate_from(frame_parser: &FrameParser) -> Result<Self, OutputError> {
        let frame_count = frame_parser.frame_count;
        let players_actor = frame_parser.players_actor.borrow();
        let players_time_series_car_data = frame_parser.players_time_series_car_data.borrow();
        let players_time_series_player_data = frame_parser.players_time_series_player_data.borrow();

        let cleaned_data = frame_parser
            .cleaned_data
            .as_ref()
            .ok_or(OutputError::MissingCleanedData)?;
        let players_time_series_boost_data = &cleaned_data.players_time_series_boost_data;
        let players_time_series_boost_pickup_data =
            &cleaned_data.players_time_series_boost_pickup_data;

        // Create player dfs
        let mut player_dfs = HashMap::new();
        for (actor_id, player_actor) in players_actor.iter() {
            if let Attribute::String(player_name) = player_actor
                .get("Engine.PlayerReplicationInfo:PlayerName")
                .expect("Could not find player name")
            {
                if let Some(time_series_car_data) = players_time_series_car_data.get(actor_id) {
                    if let Some(time_series_player_data) =
                        players_time_series_player_data.get(actor_id)
                    {
                        if let Some(time_series_boost_data) =
                            players_time_series_boost_data.get(actor_id)
                        {
                            if let Some(time_series_boost_pickup_data) =
                                players_time_series_boost_pickup_data.get(actor_id)
                            {
                                let player_df = create_player_df(
                                    time_series_car_data,
                                    time_series_player_data,
                                    time_series_boost_data,
                                    time_series_boost_pickup_data,
                                    frame_count,
                                )?;
                                player_dfs.insert(actor_id.0, player_df);
                            } else {
                                error!("Failed to write output for {} due to missing time-series boost pickup data.", player_name);
                            };
                        } else {
                            error!(
                            "Failed to write output for {} due to missing time-series boost data.",
                            player_name
                        );
                        };
                    } else {
                        error!(
                            "Failed to write output for {} due to missing time-series player data.",
                            player_name
                        );
                    };
                } else {
                    error!(
                        "Failed to write output for {} due to missing time-series car data.",
                        player_name
                    );
                };
            } else {
                error!(
                    "Failed to write output for player due to failure to parse for player name."
                );
            };
        }

        // Create ball df
        let ball_df = create_ball_df(&frame_parser.time_series_ball_data.borrow(), frame_count)?;

        // Create game df
        let game_df = create_game_df(
            &frame_parser.time_series_replay_data.borrow(),
            &frame_parser.time_series_game_event_data.borrow(),
            frame_count,
        )?;

        Ok(Self {
            game: game_df,
            ball: ball_df,
            players: player_dfs,
        })
    }
}

fn create_player_df(
    time_series_car_data: &HashMap<usize, TimeSeriesCarData>,
    time_series_player_data: &HashMap<usize, TimeSeriesPlayerData>,
    time_series_boost_data: &HashMap<usize, TimeSeriesBoostData>,
    time_series_boost_pickup_data: &HashMap<usize, Option<BoostPickupKind>>,
    frame_count: usize,
) -> Result<DataFrame, OutputError> {
    // Car data
    let mut is_sleeping: Vec<Option<bool>> = vec![None; frame_count];
    let mut pos_x: Vec<Option<f32>> = vec![None; frame_count];
    let mut pos_y: Vec<Option<f32>> = vec![None; frame_count];
    let mut pos_z: Vec<Option<f32>> = vec![None; frame_count];
    let mut vel_x: Vec<Option<f32>> = vec![None; frame_count];
    let mut vel_y: Vec<Option<f32>> = vec![None; frame_count];
    let mut vel_z: Vec<Option<f32>> = vec![None; frame_count];
    let mut rot_pitch: Vec<Option<f32>> = vec![None; frame_count];
    let mut rot_yaw: Vec<Option<f32>> = vec![None; frame_count];
    let mut rot_roll: Vec<Option<f32>> = vec![None; frame_count];
    let mut ang_vel_x: Vec<Option<f32>> = vec![None; frame_count];
    let mut ang_vel_y: Vec<Option<f32>> = vec![None; frame_count];
    let mut ang_vel_z: Vec<Option<f32>> = vec![None; frame_count];
    let mut throttle: Vec<Option<u8>> = vec![None; frame_count];
    let mut steer: Vec<Option<u8>> = vec![None; frame_count];
    let mut handbrake: Vec<Option<u8>> = vec![None; frame_count];

    // Player data
    let mut match_score: Vec<Option<i32>> = vec![None; frame_count];
    let mut match_goals: Vec<Option<i32>> = vec![None; frame_count];
    let mut match_assists: Vec<Option<i32>> = vec![None; frame_count];
    let mut match_saves: Vec<Option<i32>> = vec![None; frame_count];
    let mut match_shots: Vec<Option<i32>> = vec![None; frame_count];
    let mut team: Vec<Option<i32>> = vec![None; frame_count];
    let mut ping: Vec<Option<u8>> = vec![None; frame_count];

    // Boost data
    let mut boost_is_active: Vec<Option<bool>> = vec![None; frame_count];
    let mut boost_amount: Vec<Option<f32>> = vec![None; frame_count];

    // Boost pickup data
    let mut boost_pickup: Vec<Option<u8>> = vec![None; frame_count];

    for (frame_number, data) in time_series_car_data.iter() {
        is_sleeping[*frame_number] = data.is_sleeping;
        pos_x[*frame_number] = data.pos_x;
        pos_y[*frame_number] = data.pos_y;
        pos_z[*frame_number] = data.pos_z;
        vel_x[*frame_number] = data.vel_x;
        vel_y[*frame_number] = data.vel_y;
        vel_z[*frame_number] = data.vel_z;
        rot_pitch[*frame_number] = data.rot_pitch;
        rot_yaw[*frame_number] = data.rot_yaw;
        rot_roll[*frame_number] = data.rot_roll;
        ang_vel_x[*frame_number] = data.ang_vel_x;
        ang_vel_y[*frame_number] = data.ang_vel_y;
        ang_vel_z[*frame_number] = data.ang_vel_z;
        throttle[*frame_number] = data.throttle;
        steer[*frame_number] = data.steer;
        handbrake[*frame_number] = data.handbrake;
    }
    for (frame_number, data) in time_series_player_data.iter() {
        match_score[*frame_number] = data.match_score;
        match_goals[*frame_number] = data.match_goals;
        match_assists[*frame_number] = data.match_assists;
        match_saves[*frame_number] = data.match_saves;
        match_shots[*frame_number] = data.match_shots;
        team[*frame_number] = data.team;
        ping[*frame_number] = data.ping;
    }
    for (frame_number, data) in time_series_boost_data.iter() {
        boost_is_active[*frame_number] = data.boost_is_active;
        boost_amount[*frame_number] = data.boost_amount;
    }
    for (frame_number, _boost_pickup) in time_series_boost_pickup_data.iter() {
        match _boost_pickup {
            Some(BoostPickupKind::Full) => boost_pickup[*frame_number] = Some(2),
            Some(BoostPickupKind::Small) => boost_pickup[*frame_number] = Some(1),
            None => boost_pickup[*frame_number] = Some(0),
        };
    }

    DataFrame::new(vec![
        BooleanChunked::new_from_opt_slice("is_sleeping", &is_sleeping).into_series(),
        Float32Chunked::new_from_opt_slice("pos_x", &pos_x).into_series(),
        Float32Chunked::new_from_opt_slice("pos_y", &pos_y).into_series(),
        Float32Chunked::new_from_opt_slice("pos_z", &pos_z).into_series(),
        Float32Chunked::new_from_opt_slice("vel_x", &vel_x).into_series(),
        Float32Chunked::new_from_opt_slice("vel_y", &vel_y).into_series(),
        Float32Chunked::new_from_opt_slice("vel_z", &vel_z).into_series(),
        Float32Chunked::new_from_opt_slice("rot_pitch", &rot_pitch).into_series(),
        Float32Chunked::new_from_opt_slice("rot_yaw", &rot_yaw).into_series(),
        Float32Chunked::new_from_opt_slice("rot_roll", &rot_roll).into_series(),
        Float32Chunked::new_from_opt_slice("ang_vel_x", &ang_vel_x).into_series(),
        Float32Chunked::new_from_opt_slice("ang_vel_y", &ang_vel_y).into_series(),
        Float32Chunked::new_from_opt_slice("ang_vel_z", &ang_vel_z).into_series(),
        UInt8Chunked::new_from_opt_slice("throttle", &throttle).into_series(),
        UInt8Chunked::new_from_opt_slice("steer", &steer).into_series(),
        UInt8Chunked::new_from_opt_slice("handbrake", &handbrake).into_series(),
        Int32Chunked::new_from_opt_slice("match_score", &match_score).into_series(),
        Int32Chunked::new_from_opt_slice("match_goals", &match_goals).into_series(),
        Int32Chunked::new_from_opt_slice("match_assists", &match_assists).into_series(),
        Int32Chunked::new_from_opt_slice("match_saves", &match_saves).into_series(),
        Int32Chunked::new_from_opt_slice("match_shots", &match_shots).into_series(),
        Int32Chunked::new_from_opt_slice("team", &team).into_series(),
        UInt8Chunked::new_from_opt_slice("ping", &ping).into_series(),
        BooleanChunked::new_from_opt_slice("boost_is_active", &boost_is_active).into_series(),
        Float32Chunked::new_from_opt_slice("boost_amount", &boost_amount).into_series(),
        UInt8Chunked::new_from_opt_slice("boost_pickup", &boost_pickup).into_series(),
    ])
    .map_err(OutputError::CreateDataFrameError)
}

fn create_ball_df(
    time_series_ball_data: &HashMap<usize, TimeSeriesBallData>,
    frame_count: usize,
) -> Result<DataFrame, OutputError> {
    let mut is_sleeping: Vec<Option<bool>> = vec![None; frame_count];
    let mut pos_x: Vec<Option<f32>> = vec![None; frame_count];
    let mut pos_y: Vec<Option<f32>> = vec![None; frame_count];
    let mut pos_z: Vec<Option<f32>> = vec![None; frame_count];
    let mut vel_x: Vec<Option<f32>> = vec![None; frame_count];
    let mut vel_y: Vec<Option<f32>> = vec![None; frame_count];
    let mut vel_z: Vec<Option<f32>> = vec![None; frame_count];
    let mut rot_pitch: Vec<Option<f32>> = vec![None; frame_count];
    let mut rot_yaw: Vec<Option<f32>> = vec![None; frame_count];
    let mut rot_roll: Vec<Option<f32>> = vec![None; frame_count];
    let mut ang_vel_x: Vec<Option<f32>> = vec![None; frame_count];
    let mut ang_vel_y: Vec<Option<f32>> = vec![None; frame_count];
    let mut ang_vel_z: Vec<Option<f32>> = vec![None; frame_count];
    let mut hit_team_num: Vec<Option<u8>> = vec![None; frame_count];

    for (frame_number, data) in time_series_ball_data.iter() {
        is_sleeping[*frame_number] = data.is_sleeping;
        pos_x[*frame_number] = data.pos_x;
        pos_y[*frame_number] = data.pos_y;
        pos_z[*frame_number] = data.pos_z;
        vel_x[*frame_number] = data.vel_x;
        vel_y[*frame_number] = data.vel_y;
        vel_z[*frame_number] = data.vel_z;
        rot_pitch[*frame_number] = data.rot_pitch;
        rot_yaw[*frame_number] = data.rot_yaw;
        rot_roll[*frame_number] = data.rot_roll;
        ang_vel_x[*frame_number] = data.ang_vel_x;
        ang_vel_y[*frame_number] = data.ang_vel_y;
        ang_vel_z[*frame_number] = data.ang_vel_z;
        hit_team_num[*frame_number] = data.hit_team_num;
    }

    DataFrame::new(vec![
        BooleanChunked::new_from_opt_slice("is_sleeping", &is_sleeping).into_series(),
        Float32Chunked::new_from_opt_slice("pos_x", &pos_x).into_series(),
        Float32Chunked::new_from_opt_slice("pos_y", &pos_y).into_series(),
        Float32Chunked::new_from_opt_slice("pos_z", &pos_z).into_series(),
        Float32Chunked::new_from_opt_slice("vel_x", &vel_x).into_series(),
        Float32Chunked::new_from_opt_slice("vel_y", &vel_y).into_series(),
        Float32Chunked::new_from_opt_slice("vel_z", &vel_z).into_series(),
        Float32Chunked::new_from_opt_slice("rot_pitch", &rot_pitch).into_series(),
        Float32Chunked::new_from_opt_slice("rot_yaw", &rot_yaw).into_series(),
        Float32Chunked::new_from_opt_slice("rot_roll", &rot_roll).into_series(),
        Float32Chunked::new_from_opt_slice("ang_vel_x", &ang_vel_x).into_series(),
        Float32Chunked::new_from_opt_slice("ang_vel_y", &ang_vel_y).into_series(),
        Float32Chunked::new_from_opt_slice("ang_vel_z", &ang_vel_z).into_series(),
        UInt8Chunked::new_from_opt_slice("hit_team_num", &hit_team_num).into_series(),
    ])
    .map_err(OutputError::CreateDataFrameError)
}

fn create_game_df(
    time_series_replay_data: &HashMap<usize, TimeSeriesReplayData>,
    time_series_game_event_data: &HashMap<usize, TimeSeriesGameEventData>,
    frame_count: usize,
) -> Result<DataFrame, OutputError> {
    let mut time: Vec<Option<f32>> = vec![None; frame_count];
    let mut delta: Vec<Option<f32>> = vec![None; frame_count];

    let mut seconds_remaining: Vec<Option<i32>> = vec![None; frame_count];
    let mut replicated_game_state_time_remaining: Vec<Option<i32>> = vec![None; frame_count];
    let mut is_overtime: Vec<Option<bool>> = vec![None; frame_count];
    let mut ball_has_been_hit: Vec<Option<bool>> = vec![None; frame_count];

    for (frame_number, data) in time_series_game_event_data.iter() {
        seconds_remaining[*frame_number] = data.seconds_remaining;
        replicated_game_state_time_remaining[*frame_number] =
            data.replicated_game_state_time_remaining;
        is_overtime[*frame_number] = data.is_overtime;
        ball_has_been_hit[*frame_number] = data.ball_has_been_hit;
    }
    for (frame_number, data) in time_series_replay_data.iter() {
        time[*frame_number] = Some(data.time);
        delta[*frame_number] = Some(data.delta);
    }

    DataFrame::new(vec![
        Float32Chunked::new_from_opt_slice("time", &time).into_series(),
        Float32Chunked::new_from_opt_slice("delta", &delta).into_series(),
        Int32Chunked::new_from_opt_slice("seconds_remaining", &seconds_remaining).into_series(),
        Int32Chunked::new_from_opt_slice(
            "replicated_game_state_time_remaining",
            &replicated_game_state_time_remaining,
        )
        .into_series(),
        BooleanChunked::new_from_opt_slice("is_overtime", &is_overtime).into_series(),
        BooleanChunked::new_from_opt_slice("ball_has_been_hit", &ball_has_been_hit).into_series(),
    ])
    .map_err(OutputError::CreateDataFrameError)
}

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("FrameParser missing cleaned_data (check if clean_up method has been called)")]
    MissingCleanedData,
    #[error("Failed to create DataFrame: {0}")]
    CreateDataFrameError(PolarsError),
}
