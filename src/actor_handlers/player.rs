use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use lazy_static::lazy_static;
use log::info;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PlayerHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for PlayerHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let actor_id = actor.new_actor.actor_id;
        let attributes = actor.attributes.borrow();

        // Add time-series data
        let data = TimeSeriesPlayerData::from(actor);
        let mut players_data = self
            .frame_parser
            .players_time_series_player_data
            .borrow_mut();
        match players_data.get_mut(&actor_id) {
            Some(player_data) => {
                player_data.insert(frame_number, data);
            }
            None => {
                let mut player_data =
                    HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                player_data.insert(frame_number, data);
                players_data.insert(actor_id, player_data);
            }
        }

        // Update actor data
        let mut players_actor_data = self.frame_parser.players_actor.borrow_mut();
        players_actor_data.insert(actor_id, attributes.clone());
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

        for key in attributes.keys() {
            if PLAYER_ATTRIBUTES_PARSED.contains(key) {
                #[allow(non_snake_case)]
                let mut _PLAYER_ATTRIBUTES_NOT_FOUND = PLAYER_ATTRIBUTES_NOT_FOUND.lock().unwrap();
                if _PLAYER_ATTRIBUTES_NOT_FOUND.contains(key) {
                    _PLAYER_ATTRIBUTES_NOT_FOUND.remove(key);
                }
            } else {
                // Unparsed attribute
                #[allow(non_snake_case)]
                let mut _PLAYER_ATTRIBUTES_NOT_PARSED =
                    PLAYER_ATTRIBUTES_NOT_PARSED.lock().unwrap();
                _PLAYER_ATTRIBUTES_NOT_PARSED.insert(key.to_string());
            }
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

lazy_static! {
    static ref PLAYER_ATTRIBUTES_PARSED: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert("TAGame.PRI_TA:MatchScore".to_string());
        set.insert("TAGame.PRI_TA:MatchGoals".to_string());
        set.insert("TAGame.PRI_TA:MatchAssists".to_string());
        set.insert("TAGame.PRI_TA:MatchSaves".to_string());
        set.insert("TAGame.PRI_TA:MatchShots".to_string());
        set.insert("Engine.PlayerReplicationInfo:Team".to_string());
        // set.insert("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera".to_string());
        set
    };
    static ref PLAYER_ATTRIBUTES_NOT_FOUND: Mutex<HashSet<String>> = {
        let mut set = HashSet::new();
        set.insert("TAGame.PRI_TA:MatchScore".to_string());
        set.insert("TAGame.PRI_TA:MatchGoals".to_string());
        set.insert("TAGame.PRI_TA:MatchAssists".to_string());
        set.insert("TAGame.PRI_TA:MatchSaves".to_string());
        set.insert("TAGame.PRI_TA:MatchShots".to_string());
        set.insert("Engine.PlayerReplicationInfo:Team".to_string());
        // set.insert("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera".to_string());
        Mutex::new(set)
    };
    static ref PLAYER_ATTRIBUTES_NOT_PARSED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub fn print_player_attributes() {
    info!("### Player attributes debug info ###");
    info!("Player attributes not found:");
    info!("{:?}", PLAYER_ATTRIBUTES_NOT_FOUND.lock().unwrap());
    info!("Player attributes not parsed");
    info!("{:?}", PLAYER_ATTRIBUTES_NOT_PARSED.lock().unwrap());
}
