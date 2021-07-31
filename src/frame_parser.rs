use crate::actor_handlers::{
    ActorHandler, ActorHandlerFactory, ActorHandlerPriority, DemoData, TeamData,
    TimeSeriesBallData, TimeSeriesBoostData, TimeSeriesCarData, TimeSeriesGameEventData,
    TimeSeriesPlayerData, WrappedUniqueId,
};
use crate::cleaner::{BoostPickupKind, BoostPickupKindCalculationError};
use crate::replay_properties_to_hash_map;
use boxcars::{ActorId, Attribute, HeaderProp, NewActor, Replay, UpdatedAttribute};
use indicatif::ProgressBar;
use indicatif::ProgressIterator;
use log::{info, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Iterator;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FrameParser {
    pub replay_version: i32,
    pub frame_count: usize,
    pub car_ids_to_player_ids: RefCell<HashMap<ActorId, ActorId>>,
    pub players_wrapped_unique_id: RefCell<HashMap<ActorId, WrappedUniqueId>>,

    pub players_actor: RefCell<HashMap<WrappedUniqueId, HashMap<String, Attribute>>>,
    pub teams_data: RefCell<HashMap<ActorId, TeamData>>,

    pub game_info_actor: RefCell<Option<HashMap<String, Attribute>>>,
    pub game_event_actor: RefCell<Option<HashMap<String, Attribute>>>,

    pub time_series_replay_data: RefCell<HashMap<usize, TimeSeriesReplayData>>,
    pub time_series_game_event_data: RefCell<HashMap<usize, TimeSeriesGameEventData>>,
    pub time_series_ball_data: RefCell<HashMap<usize, TimeSeriesBallData>>,
    pub players_time_series_car_data:
        RefCell<HashMap<WrappedUniqueId, HashMap<usize, TimeSeriesCarData>>>,
    pub players_time_series_player_data:
        RefCell<HashMap<WrappedUniqueId, HashMap<usize, TimeSeriesPlayerData>>>,
    pub players_time_series_boost_data:
        RefCell<HashMap<WrappedUniqueId, HashMap<usize, TimeSeriesBoostData>>>,
    pub demos_data: RefCell<Vec<DemoData>>,

    pub cleaned_data: Option<CleanedData>,
}

impl FrameParser {
    pub fn from_replay(replay: &Replay, show_progress: bool) -> Result<Self, FrameParserError> {
        let mut frame_parser = Self::new(replay);
        frame_parser.process_replay(replay, show_progress)?;
        frame_parser
            .clean_up()
            .map_err(FrameParserError::CleanUpError)?;
        Ok(frame_parser)
    }

    pub fn new(replay: &Replay) -> Self {
        let properties = replay_properties_to_hash_map(replay);
        let replay_version = match properties.get("ReplayVersion") {
            Some(HeaderProp::Int(replay_version)) => replay_version,
            _ => panic!("Cannot parse replay version from header properties."),
        };
        match &replay.network_frames {
            Some(network_frames) => {
                let frame_count = network_frames.frames.len();
                info!("Replay with {} frames", frame_count);

                Self {
                    replay_version: *replay_version,
                    frame_count,
                    car_ids_to_player_ids: RefCell::new(HashMap::new()),
                    players_wrapped_unique_id: RefCell::new(HashMap::new()),

                    teams_data: RefCell::new(HashMap::new()),
                    players_actor: RefCell::new(HashMap::new()),
                    game_info_actor: RefCell::new(None),
                    game_event_actor: RefCell::new(None),

                    time_series_replay_data: RefCell::new(HashMap::with_capacity(frame_count)),
                    time_series_game_event_data: RefCell::new(HashMap::with_capacity(frame_count)),
                    time_series_ball_data: RefCell::new(HashMap::with_capacity(frame_count)),
                    players_time_series_car_data: RefCell::new(HashMap::new()),
                    players_time_series_player_data: RefCell::new(HashMap::new()),
                    players_time_series_boost_data: RefCell::new(HashMap::new()),
                    demos_data: RefCell::new(vec![]),

                    cleaned_data: None,
                }
            }
            None => {
                panic!("No network frames.")
            }
        }
    }

    pub fn process_replay(
        &self,
        replay: &Replay,
        show_progress: bool,
    ) -> Result<(), FrameParserError> {
        let network_frames = replay
            .network_frames
            .as_ref()
            .ok_or(FrameParserError::MissingNetworkFrames)?;

        let handler_factory = ActorHandlerFactory::new(self);
        let mut actor_handlers: HashMap<
            ActorHandlerPriority,
            HashMap<ActorId, Box<dyn ActorHandler>>,
        > = HashMap::new();
        let mut actors: HashMap<ActorId, Actor> = HashMap::new();
        let replay_objects = &replay.objects;

        let mut time_series_replay_data = self.time_series_replay_data.borrow_mut();

        let iter: Box<dyn Iterator<Item = (usize, &boxcars::Frame)>> = if show_progress {
            let progress_bar = ProgressBar::new(self.frame_count as u64);
            progress_bar.set_draw_rate(30);
            Box::new(
                network_frames
                    .frames
                    .iter()
                    .enumerate()
                    .progress_with(progress_bar),
            )
        } else {
            Box::new(network_frames.frames.iter().enumerate())
        };
        for (frame_number, frame) in iter.into_iter() {
            let time = frame.time;
            let delta = frame.delta;
            // info!("### Frame {} ({}, {})", frame_number, time, delta);

            // Handle deleted actors first
            for deleted_actor_id in &frame.deleted_actors {
                if actors.remove(&deleted_actor_id).is_none() {
                    warn!(
                        "Could not find actor {} to delete on frame {}.",
                        deleted_actor_id, frame_number
                    );
                }
                if let Some(_actor_handlers) = actor_handlers.get_mut(&ActorHandlerPriority::First)
                {
                    _actor_handlers.remove(deleted_actor_id);
                }
                if let Some(_actor_handlers) =
                    actor_handlers.get_mut(&ActorHandlerPriority::Standard)
                {
                    _actor_handlers.remove(deleted_actor_id);
                }
            }

            // Handle new actors
            for new_actor in &frame.new_actors {
                let actor_id = new_actor.actor_id;
                actors.insert(actor_id, Actor::new(new_actor));
                if let Some(handler) =
                    handler_factory.get_handler(new_actor.object_id, &replay_objects)
                {
                    let _actor_handlers = actor_handlers
                        .entry(handler.priority())
                        .or_insert_with(HashMap::new);
                    _actor_handlers.insert(actor_id, handler);
                }
            }

            // Handle updated actors
            for updated_attribute in &frame.updated_actors {
                let actor_id = updated_attribute.actor_id;
                let actor = actors
                    .get_mut(&actor_id)
                    .expect("Updated actor does not exist.");
                actor.update_attribute(updated_attribute, &replay_objects);
            }

            // Stop data collection after goal
            // if previous_goal_frame_number.is_some()
            //     && frame_number > previous_goal_frame_number.unwrap()
            // {
            //     // TODO: Set players to sleeping
            //     // https://github.com/SaltieRL/carball/blob/f9e4854e173bb6db3e53cc93ac1daa4e58952e69/carball/json_parser/frame_parser.py#L189
            // }

            // Run handler updates
            for priority in ActorHandlerPriority::iterator() {
                if let Some(mut _actor_handlers) = actor_handlers.get_mut(priority) {
                    for (actor_id, handler) in _actor_handlers.iter_mut() {
                        handler.update(
                            actors.get(actor_id).ok_or_else(|| {
                                FrameParserError::ActorUpdateMissingIdError(frame_number, *actor_id)
                            })?,
                            frame_number,
                            time,
                            delta,
                        )
                    }
                }
            }
            time_series_replay_data.insert(frame_number, TimeSeriesReplayData { time, delta });
        }

        Ok(())
    }

    pub fn clean_up(&mut self) -> Result<(), BoostPickupKindCalculationError> {
        let players_actor = self.players_actor.borrow();

        let players_time_series_car_data = self.players_time_series_car_data.borrow();
        let players_time_series_boost_data = self.players_time_series_boost_data.borrow();

        let mut cleaned_data = CleanedData::new();

        for (wrapped_unique_id, player_actor) in players_actor.iter() {
            let player_name = match player_actor.get("Engine.PlayerReplicationInfo:PlayerName") {
                Some(Attribute::String(_player_name)) => _player_name,
                _ => "UnknownName",
            };
            // if player_name == "AyyJayy" {
            //     dbg!(&player_actor);
            // }
            // let player_wrapped_unique_id = players_wrapped_unique_id.get(&actor_id).unwrap();

            if let Some(time_series_boost_data) =
                players_time_series_boost_data.get(wrapped_unique_id)
            {
                if let Some(time_series_car_data) =
                    players_time_series_car_data.get(wrapped_unique_id)
                {
                    let mut cleaned_time_series_boost_data =
                        HashMap::with_capacity(self.frame_count);
                    let mut cleaned_time_series_boost_pickup_data =
                        HashMap::with_capacity(self.frame_count);

                    let mut last_raw_boost_amount: f32 = 0.0;
                    let mut last_predicted_boost_amount: f32 = 0.0;
                    let mut delta_boost_is_active_since_last_update: f32 = 0.0;
                    for frame_number in 0..(self.frame_count - 1) {
                        if let Some(boost_data) = time_series_boost_data.get(&frame_number) {
                            let delta = self
                                .time_series_replay_data
                                .borrow()
                                .get(&frame_number)
                                .unwrap()
                                .delta;
                            let new_raw_boost_amount = boost_data.boost_amount.unwrap_or(0.0);
                            if (last_raw_boost_amount - new_raw_boost_amount).abs() > f32::EPSILON {
                                // Boost amount updated in replay file
                                // Detect boost pickup
                                if new_raw_boost_amount > last_predicted_boost_amount {
                                    cleaned_time_series_boost_pickup_data.insert(
                                        frame_number,
                                        BoostPickupKind::detect_boost_pickup_kind(
                                            last_predicted_boost_amount,
                                            new_raw_boost_amount,
                                            time_series_car_data.get(&frame_number),
                                            frame_number,
                                            player_name,
                                        )?,
                                    );
                                }

                                // Reset delta counter as boost amount is up-to-date and accurate.
                                delta_boost_is_active_since_last_update = 0.0;
                                last_raw_boost_amount = new_raw_boost_amount;
                            } else {
                                cleaned_time_series_boost_pickup_data.insert(frame_number, None);
                            }
                            if boost_data.boost_is_active.unwrap_or(false) {
                                if delta_boost_is_active_since_last_update == 0.0 {
                                    // Don't add full delta on first frame to correct for frame interval.
                                    // I.e. Person is likely not holding the boost button for entire duration of delta (on first and final frames where boost is active).
                                    // Corrects for (but does not completely fix) the observed boost usage overestimation.
                                    delta_boost_is_active_since_last_update += delta / 2.0;
                                } else {
                                    delta_boost_is_active_since_last_update += delta;
                                }
                                let predicted_boost_amount = new_raw_boost_amount
                                    - (33.3 * delta_boost_is_active_since_last_update);
                                last_predicted_boost_amount = predicted_boost_amount;

                                let mut modified_boost_data = *boost_data;
                                modified_boost_data.boost_amount =
                                    Some(predicted_boost_amount.clamp(0.0, 100.0));
                                cleaned_time_series_boost_data
                                    .insert(frame_number, modified_boost_data);
                            } else {
                                cleaned_time_series_boost_data.insert(frame_number, *boost_data);
                                last_predicted_boost_amount = new_raw_boost_amount;
                            }
                        }
                    }
                    cleaned_data
                        .players_time_series_boost_data
                        .insert(wrapped_unique_id.clone(), cleaned_time_series_boost_data);
                    cleaned_data.players_time_series_boost_pickup_data.insert(
                        wrapped_unique_id.clone(),
                        cleaned_time_series_boost_pickup_data,
                    );
                }
            }
        }
        self.cleaned_data = Some(cleaned_data);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Actor<'a> {
    pub new_actor: &'a NewActor,
    pub attributes: RefCell<HashMap<String, Attribute>>,
}

impl<'a> Actor<'a> {
    fn new(new_actor: &'a NewActor) -> Self {
        Self {
            new_actor,
            attributes: RefCell::new(HashMap::new()),
        }
    }

    fn update_attribute(
        &mut self,
        updated_attribute: &UpdatedAttribute,
        replay_objects: &[String],
    ) {
        let prop_name = replay_objects[updated_attribute.object_id.0 as usize].clone();
        let mut attributes = self.attributes.borrow_mut();
        attributes.insert(prop_name, updated_attribute.attribute.clone());
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesReplayData {
    pub time: f32,
    pub delta: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CleanedData {
    pub players_actor: HashMap<WrappedUniqueId, HashMap<String, Attribute>>,
    pub players_time_series_boost_data:
        HashMap<WrappedUniqueId, HashMap<usize, TimeSeriesBoostData>>,
    pub players_time_series_boost_pickup_data:
        HashMap<WrappedUniqueId, HashMap<usize, Option<BoostPickupKind>>>,
}

impl CleanedData {
    pub fn new() -> Self {
        Self {
            players_actor: HashMap::new(),
            players_time_series_boost_data: HashMap::new(),
            players_time_series_boost_pickup_data: HashMap::new(),
        }
    }
}

impl Default for CleanedData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Error)]
pub enum FrameParserError {
    #[error("missing network frames from parsed replay")]
    MissingNetworkFrames,
    #[error("trying to update missing actor {1} on frame {0}")]
    ActorUpdateMissingIdError(usize, ActorId),
    #[error("clean up failed: {0}")]
    CleanUpError(BoostPickupKindCalculationError),
}
