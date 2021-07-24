use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BoostPickupHandler<'a> {
    frame_parser: &'a FrameParser,
    _last_picked_up_int: u8,
}

impl<'a> ActorHandler<'a> for BoostPickupHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self {
            frame_parser,
            _last_picked_up_int: 0,
        }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let attributes = actor.attributes.borrow();

        if let Some(Attribute::PickupNew(pickup_new)) =
            attributes.get("TAGame.VehiclePickup_TA:NewReplicatedPickupData")
        {
            if let Some(car_actor_id) = pickup_new.instigator {
                let car_ids_to_player_ids = self.frame_parser.car_ids_to_player_ids.borrow();
                if let Some(player_actor_id) = car_ids_to_player_ids.get(&car_actor_id) {
                    if pickup_new.picked_up != self._last_picked_up_int {
                        self._last_picked_up_int = pickup_new.picked_up;
                        let mut players_data = self
                            .frame_parser
                            .players_time_series_boost_pickup_data
                            .borrow_mut();
                        match players_data.get_mut(&player_actor_id) {
                            Some(player_data) => {
                                player_data.insert(frame_number, true);
                            }
                            None => {
                                let mut player_data = HashMap::with_capacity(
                                    self.frame_parser.frame_count - frame_number,
                                );
                                player_data.insert(frame_number, true);
                                players_data.insert(*player_actor_id, player_data);
                            }
                        }
                    }
                }
            };
        } else if let Some(Attribute::Pickup(pickup)) =
            attributes.get("TAGame.VehiclePickup_TA:ReplicatedPickupData")
        {
            if let Some(car_actor_id) = pickup.instigator {
                let car_ids_to_player_ids = self.frame_parser.car_ids_to_player_ids.borrow();
                if let Some(player_actor_id) = car_ids_to_player_ids.get(&car_actor_id) {
                    let mut players_data = self
                        .frame_parser
                        .players_time_series_boost_pickup_data
                        .borrow_mut();
                    match players_data.get_mut(&player_actor_id) {
                        Some(player_data) => {
                            player_data.insert(frame_number, pickup.picked_up);
                        }
                        None => {
                            let mut player_data = HashMap::with_capacity(
                                self.frame_parser.frame_count - frame_number,
                            );
                            player_data.insert(frame_number, pickup.picked_up);
                            players_data.insert(*player_actor_id, player_data);
                        }
                    }
                }
            };
        }
    }
}
