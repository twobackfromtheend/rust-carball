use crate::replay_properties_to_hash_map;
use boxcars::{HeaderProp, Replay};
use log::error;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Game {
    pub id: String,
    pub replay_version: i32,
    pub num_frames: i32,
    pub replay_name: Option<String>,
    pub map_name: Option<String>,
    pub date: Option<String>,
    pub match_type: Option<String>,
    pub team_0_score: Option<i32>,
    pub team_1_score: Option<i32>,
    pub goals: Vec<Goal>,
}

impl Game {
    pub fn from(replay: &Replay) -> Self {
        let properties = replay_properties_to_hash_map(replay);

        match properties.get("Id") {
            Some(HeaderProp::Str(id)) => match properties.get("ReplayVersion") {
                Some(HeaderProp::Int(replay_version)) => match properties.get("NumFrames") {
                    Some(HeaderProp::Int(num_frames)) => {
                        let mut replay_name = None;
                        let mut map_name = None;
                        let mut date = None;
                        let mut match_type = None;
                        let mut team_0_score = None;
                        let mut team_1_score = None;

                        if let Some(HeaderProp::Str(_replay_name)) = properties.get("ReplayName") {
                            replay_name = Some(_replay_name.to_string());
                        }
                        if let Some(HeaderProp::Name(_map_name)) = properties.get("MapName") {
                            map_name = Some(_map_name.to_string());
                        }
                        if let Some(HeaderProp::Str(_date)) = properties.get("Date") {
                            date = Some(_date.to_string());
                        }
                        if let Some(HeaderProp::Name(_match_type)) = properties.get("MatchType") {
                            match_type = Some(_match_type.to_string());
                        }
                        if let Some(HeaderProp::Int(_team_0_score)) = properties.get("Team0Score") {
                            team_0_score = Some(*_team_0_score);
                        }
                        if let Some(HeaderProp::Int(_team_1_score)) = properties.get("Team1Score") {
                            team_1_score = Some(*_team_1_score);
                        }

                        Self {
                            id: id.to_string(),
                            replay_version: *replay_version,
                            num_frames: *num_frames,
                            replay_name,
                            map_name,
                            date,
                            match_type,
                            team_0_score,
                            team_1_score,
                            goals: Goal::from_replay_properties(&properties),
                        }
                    }
                    Some(_) => {
                        panic!("replay header's NumFrames property has unexpected type");
                    }
                    None => {
                        panic!("replay header has no NumFrames property");
                    }
                },
                Some(_) => {
                    panic!("replay header's ReplayVersion property has unexpected type");
                }
                None => {
                    panic!("replay header has no ReplayVersion property");
                }
            },
            Some(_) => {
                panic!("replay header's Id property has unexpected type");
            }
            None => {
                panic!("replay header has no Id property");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Goal {
    pub frame: i32,
    pub player_name: String,
    pub is_orange: bool,
}

impl Goal {
    pub fn from_replay_properties(properties: &HashMap<&str, &HeaderProp>) -> Vec<Self> {
        match properties.get("Goals") {
            Some(HeaderProp::Array(goals)) => goals.iter().map(|g| Goal::from(g)).collect(),
            _ => {
                error!("Failed to parse Goals key in replay properties.");
                vec![]
            }
        }
    }

    pub fn from(data: &[(String, HeaderProp)]) -> Self {
        let goal_properties: HashMap<String, HeaderProp> = data.iter().cloned().collect();
        Goal {
            frame: match goal_properties.get("frame") {
                Some(HeaderProp::Int(frame)) => *frame,
                _ => {
                    error!("Could not find frame for goal.");
                    0
                }
            },
            player_name: match goal_properties.get("PlayerName") {
                Some(HeaderProp::Str(player_name)) => player_name.to_string(),
                _ => {
                    error!("Could not find player name for goal.");
                    "".to_string()
                }
            },
            is_orange: match goal_properties.get("PlayerTeam") {
                Some(HeaderProp::Int(team)) => *team == 1,
                _ => {
                    error!("Could not find player team for goal.");
                    false
                }
            },
        }
    }
}
