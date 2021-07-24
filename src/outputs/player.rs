use crate::actor_handlers::TeamData;
use crate::frame_parser::FrameParser;
use boxcars::attributes::RemoteId;
use boxcars::{ActorId, Attribute};
use log::error;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Player {
    pub _actor_id: i32,
    pub name: String,
    pub online_id: Option<String>,
    pub online_id_kind: Option<String>,
    pub is_orange: Option<bool>,
    pub match_score: i32,
    pub match_goals: i32,
    pub match_assists: i32,
    pub match_saves: i32,
    pub match_shots: i32,
}

impl Player {
    pub fn from_frame_parser(frame_parser: &FrameParser) -> Vec<Self> {
        let players_actor = frame_parser.players_actor.borrow();
        let teams_actor = frame_parser.teams_data.borrow(); // TODO: REMOVE
        players_actor
            .iter()
            .map(|(actor_id, player_actor)| Player::from(actor_id, player_actor, &teams_actor))
            .collect()
    }

    pub fn from(
        actor_id: &boxcars::ActorId,
        attributes: &HashMap<String, boxcars::Attribute>,
        teams_actor: &HashMap<ActorId, TeamData>,
    ) -> Self {
        Self {
            _actor_id: actor_id.0,
            name: match attributes.get("Engine.PlayerReplicationInfo:PlayerName") {
                Some(Attribute::String(name)) => name.to_string(),
                _ => {
                    error!("Could not find name for player {:?}", attributes);
                    "".to_string()
                }
            },
            online_id: match attributes.get("Engine.PlayerReplicationInfo:UniqueId") {
                Some(Attribute::UniqueId(unique_id)) => {
                    Some(match &unique_id.remote_id {
                        RemoteId::PlayStation(id) => id.online_id.to_string(),
                        RemoteId::Steam(id) => id.to_string(),
                        RemoteId::Switch(id) => id.online_id.to_string(),
                        RemoteId::Epic(id) => id.to_string(),
                        RemoteId::Xbox(id) => id.to_string(),
                        // TODO: Figure out more details regarding these RemoteId types.
                        RemoteId::PsyNet(id) => id.online_id.to_string(),
                        RemoteId::SplitScreen(id) => id.to_string(),
                        RemoteId::QQ(id) => id.to_string(),
                    })
                }
                _ => None,
            },
            online_id_kind: match attributes.get("Engine.PlayerReplicationInfo:UniqueId") {
                Some(Attribute::UniqueId(unique_id)) => {
                    Some(match &unique_id.remote_id {
                        RemoteId::PlayStation(_) => "PlayStation".to_string(),
                        RemoteId::Steam(_) => "Steam".to_string(),
                        RemoteId::Switch(_) => "Switch".to_string(),
                        RemoteId::Epic(_) => "Epic".to_string(),
                        RemoteId::Xbox(_) => "Xbox".to_string(),
                        // TODO: Figure out more details regarding these RemoteId types.
                        RemoteId::PsyNet(_) => "PsyNet".to_string(),
                        RemoteId::SplitScreen(_) => "SplitScreen".to_string(),
                        RemoteId::QQ(_) => "QQ".to_string(),
                    })
                }
                _ => None,
            },
            is_orange: match attributes.get("Engine.PlayerReplicationInfo:Team") {
                Some(Attribute::ActiveActor(team_active_actor)) => {
                    let team_actor_id = team_active_actor.actor;
                    teams_actor
                        .get(&team_actor_id)
                        .map(|team_data| team_data.is_orange)
                }
                _ => None,
            },
            match_score: match attributes.get("TAGame.PRI_TA:MatchScore") {
                Some(Attribute::Int(match_score)) => *match_score,
                _ => 0,
            },
            match_goals: match attributes.get("TAGame.PRI_TA:MatchGoals") {
                Some(Attribute::Int(match_goals)) => *match_goals,
                _ => 0,
            },
            match_assists: match attributes.get("TAGame.PRI_TA:MatchAssists") {
                Some(Attribute::Int(match_assists)) => *match_assists,
                _ => 0,
            },
            match_saves: match attributes.get("TAGame.PRI_TA:MatchSaves") {
                Some(Attribute::Int(match_saves)) => *match_saves,
                _ => 0,
            },
            match_shots: match attributes.get("TAGame.PRI_TA:MatchShots") {
                Some(Attribute::Int(match_shots)) => *match_shots,
                _ => 0,
            },
        }
    }
}
