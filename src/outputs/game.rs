use boxcars::HeaderProp;
use boxcars::Replay;
use log::error;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Game {
    id: Option<String>,
    replay_name: Option<String>,
    map_name: Option<String>,
    date: Option<String>,
    match_type: Option<String>,
    team_0_score: Option<i32>,
    team_1_score: Option<i32>,
    goals: Vec<Goal>,
}

impl Game {
    pub fn from(replay: &Replay) -> Self {
        // Convert from Vec to HashMap. boxcars uses a Vec to allow for potential duplicate keys.
        let properties: HashMap<String, HeaderProp> =
            replay.properties.clone().into_iter().collect();

        let mut id = None;
        let mut replay_name = None;
        let mut map_name = None;
        let mut date = None;
        let mut match_type = None;
        let mut team_0_score = None;
        let mut team_1_score = None;

        if let Some(HeaderProp::Str(_id)) = properties.get("Id") {
            id = Some(_id.to_string());
        }
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
            id,
            replay_name,
            map_name,
            date,
            match_type,
            team_0_score,
            team_1_score,
            goals: Goal::from_replay_properties(&properties),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Goal {
    frame: i32,
    player_name: String,
    is_orange: bool,
}

impl Goal {
    pub fn from_replay_properties(properties: &HashMap<String, HeaderProp>) -> Vec<Self> {
        match properties.get("Goals") {
            Some(HeaderProp::Array(goals)) => goals.iter().map(|g| Goal::from(g)).collect(),
            _ => {
                error!("Failed to parse Goals key in replay properties.");
                vec![]
            }
        }
    }

    pub fn from(data: &[(String, HeaderProp)]) -> Self {
        let goal_properties: HashMap<String, HeaderProp> = data.to_owned().into_iter().collect();
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
