use crate::actor_handlers::{ActorHandler, ActorHandlerPriority, WrappedUniqueId};
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlayerHandler<'a> {
    frame_parser: &'a FrameParser,
    wrapped_unique_id: Option<WrappedUniqueId>,
}

impl<'a> ActorHandler<'a> for PlayerHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self {
            frame_parser,
            wrapped_unique_id: None,
        }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let actor_id = actor.new_actor.actor_id;
        let attributes = actor.attributes.borrow();

        if self.wrapped_unique_id.is_none() {
            let wrapped_unique_id = WrappedUniqueId::from(&attributes);
            self.wrapped_unique_id = Some(wrapped_unique_id.clone());
            let mut players_wrapped_unique_id =
                self.frame_parser.players_wrapped_unique_id.borrow_mut();
            players_wrapped_unique_id.insert(actor_id, wrapped_unique_id);
        }

        // Add time-series data
        let data = TimeSeriesPlayerData::from(actor);
        let mut players_data = self
            .frame_parser
            .players_time_series_player_data
            .borrow_mut();
        let wrapped_unique_id = self.wrapped_unique_id.as_ref().unwrap().clone();
        match players_data.get_mut(&wrapped_unique_id) {
            Some(player_data) => {
                player_data.insert(frame_number, data);
            }
            None => {
                let mut player_data =
                    HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                player_data.insert(frame_number, data);
                players_data.insert(wrapped_unique_id.clone(), player_data);
            }
        }

        // Record team every 500 frames.
        if frame_number > 500 && frame_number % 500 == 1 {
            let teams_actor = self.frame_parser.teams_data.borrow();
            let mut players_teams = self.frame_parser.players_teams.borrow_mut();
            if let Some(Attribute::ActiveActor(team_active_actor)) =
                attributes.get("Engine.PlayerReplicationInfo:Team")
            {
                let team_actor_id = team_active_actor.actor;
                if let Some(is_orange) = teams_actor
                    .get(&team_actor_id)
                    .map(|team_data| team_data.is_orange)
                {
                    let player_teams = players_teams
                        .entry(wrapped_unique_id.clone())
                        .or_default();
                    player_teams
                        .entry(is_orange)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            };
        }

        // Update actor data (only on final frame to avoid unnecessary cloning)
        if frame_number == self.frame_parser.frame_count - 1 {
            let mut players_actor_data = self.frame_parser.players_actor.borrow_mut();
            if !players_actor_data.contains_key(&wrapped_unique_id) {
                players_actor_data.insert(wrapped_unique_id, attributes.clone());
            } else {
                let match_score = match attributes.get("TAGame.PRI_TA:MatchScore") {
                    Some(Attribute::Int(match_score)) => *match_score,
                    _ => 0,
                };
                let existing_player_actor_data =
                    players_actor_data.get(&wrapped_unique_id).unwrap();
                let existing_match_score =
                    match existing_player_actor_data.get("TAGame.PRI_TA:MatchScore") {
                        Some(Attribute::Int(match_score)) => *match_score,
                        _ => 0,
                    };
                if match_score > existing_match_score {
                    // Replace existing entry with this, as this has higher match score.
                    // dbg!(&existing_player_actor_data);
                    // dbg!(&players_actor_data);
                    players_actor_data.insert(wrapped_unique_id, attributes.clone());
                }
            }
        }
    }

    fn priority(&self) -> ActorHandlerPriority {
        ActorHandlerPriority::First
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSeriesPlayerData {
    pub match_score: Option<i32>,
    pub match_goals: Option<i32>,
    pub match_assists: Option<i32>,
    pub match_saves: Option<i32>,
    pub match_shots: Option<i32>,
    pub team: Option<i32>,
    pub ping: Option<u8>,
}

impl TimeSeriesPlayerData {
    fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();

        let mut match_score = None;
        let mut match_goals = None;
        let mut match_assists = None;
        let mut match_saves = None;
        let mut match_shots = None;
        let mut team = None;
        let mut ping = None;

        if let Some(Attribute::Int(_match_score)) = attributes.get("TAGame.PRI_TA:MatchScore") {
            match_score = Some(*_match_score);
        }
        if let Some(Attribute::Int(_match_goals)) = attributes.get("TAGame.PRI_TA:MatchGoals") {
            match_goals = Some(*_match_goals);
        }
        if let Some(Attribute::Int(_match_assists)) = attributes.get("TAGame.PRI_TA:MatchAssists") {
            match_assists = Some(*_match_assists);
        }
        if let Some(Attribute::Int(_match_saves)) = attributes.get("TAGame.PRI_TA:MatchSaves") {
            match_saves = Some(*_match_saves);
        }
        if let Some(Attribute::Int(_match_shots)) = attributes.get("TAGame.PRI_TA:MatchShots") {
            match_shots = Some(*_match_shots);
        }
        if let Some(Attribute::ActiveActor(_team)) =
            attributes.get("Engine.PlayerReplicationInfo:Team")
        {
            team = Some(_team.actor.0);
        }
        if let Some(Attribute::Byte(_ping)) = attributes.get("Engine.PlayerReplicationInfo:Ping") {
            ping = Some(*_ping);
        }

        TimeSeriesPlayerData {
            match_score,
            match_goals,
            match_assists,
            match_saves,
            match_shots,
            team,
            ping,
        }
    }
}
