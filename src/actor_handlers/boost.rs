use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BoostHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for BoostHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let attributes = actor.attributes.borrow();

        if let Some(Attribute::ActiveActor(active_actor)) =
            attributes.get("TAGame.CarComponent_TA:Vehicle")
        {
            let car_actor_id = active_actor.actor;
            let car_ids_to_player_ids = self.frame_parser.car_ids_to_player_ids.borrow();
            if let Some(player_actor_id) = car_ids_to_player_ids.get(&car_actor_id) {
                let boost_data = TimeSeriesBoostData::from(actor);
                let mut players_data = self
                    .frame_parser
                    .players_time_series_boost_data
                    .borrow_mut();

                let players_wrapped_unique_id =
                    self.frame_parser.players_wrapped_unique_id.borrow();
                let player_wrapped_unique_id =
                    players_wrapped_unique_id.get(player_actor_id).unwrap();
                match players_data.get_mut(player_wrapped_unique_id) {
                    Some(player_data) => {
                        player_data.insert(frame_number, boost_data);
                    }
                    None => {
                        let mut player_data =
                            HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                        player_data.insert(frame_number, boost_data);
                        players_data.insert(player_wrapped_unique_id.clone(), player_data);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesBoostData {
    pub boost_is_active: Option<bool>,
    pub boost_amount: Option<f32>,
}

impl TimeSeriesBoostData {
    pub fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();

        let mut boost_is_active = None;
        let mut boost_amount = None;
        if let Some(Attribute::Byte(_boost_is_active_int)) =
            attributes.get("TAGame.CarComponent_TA:ReplicatedActive")
        {
            boost_is_active = Some(*_boost_is_active_int & 1 != 0); // Boost is active when the integer is odd.
        }
        if let Some(Attribute::Byte(_boost_amount)) =
            attributes.get("TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount")
        {
            boost_amount = Some(*_boost_amount as f32 / 2.55);
        }
        TimeSeriesBoostData {
            boost_is_active,
            boost_amount,
        }
    }
}
