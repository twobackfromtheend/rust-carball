use crate::actor_handlers::{ActorHandler, RigidBodyData, WrappedUniqueId};
use crate::frame_parser::{Actor, FrameParser};
use boxcars::attributes::Demolish;
use boxcars::{ActorId, Attribute};
use log::error;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CarHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for CarHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let car_actor_id = actor.new_actor.actor_id;
        let mut attributes = actor.attributes.borrow_mut();
        if let Some(Attribute::ActiveActor(active_actor)) =
            attributes.get("Engine.Pawn:PlayerReplicationInfo")
        {
            let player_actor_id = active_actor.actor;
            let players_wrapped_unique_id = self.frame_parser.players_wrapped_unique_id.borrow();
            if let Some(player_wrapped_unique_id) = players_wrapped_unique_id.get(&player_actor_id)
            {
                // Assign car-player links
                let mut car_ids_to_player_ids =
                    self.frame_parser.car_ids_to_player_ids.borrow_mut();
                car_ids_to_player_ids.insert(car_actor_id, player_actor_id);

                // Add time-series car data
                let car_data =
                    TimeSeriesCarData::from(actor, &attributes, self.frame_parser.replay_version); // attributes passed here as borrowed mut above.
                let mut players_data = self.frame_parser.players_time_series_car_data.borrow_mut();
                match players_data.get_mut(player_wrapped_unique_id) {
                    Some(player_data) => {
                        player_data.insert(frame_number, car_data);
                    }
                    None => {
                        let mut player_data =
                            HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                        player_data.insert(frame_number, car_data);
                        players_data.insert(player_wrapped_unique_id.clone(), player_data);
                    }
                }
            }
        }

        // Add demos
        if let Some(Attribute::Demolish(demolish)) =
            attributes.get("TAGame.Car_TA:ReplicatedDemolish")
        {
            let players_wrapped_unique_id = self.frame_parser.players_wrapped_unique_id.borrow();
            let car_ids_to_player_ids = self.frame_parser.car_ids_to_player_ids.borrow();
            let mut demos_data = self.frame_parser.demos_data.borrow_mut();
            if let Some(demo_data) = DemoData::from(
                demolish,
                frame_number,
                &car_ids_to_player_ids,
                &players_wrapped_unique_id,
            ) {
                demos_data.push(demo_data);
            }
            attributes.remove("TAGame.Car_TA:ReplicatedDemolish");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesCarData {
    pub throttle: Option<u8>,
    pub steer: Option<u8>,
    pub handbrake: Option<u8>,
    pub is_sleeping: Option<bool>,
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub pos_z: Option<f32>,
    pub vel_x: Option<f32>,
    pub vel_y: Option<f32>,
    pub vel_z: Option<f32>,
    pub quat_w: Option<f32>,
    pub quat_x: Option<f32>,
    pub quat_y: Option<f32>,
    pub quat_z: Option<f32>,
    pub ang_vel_x: Option<f32>,
    pub ang_vel_y: Option<f32>,
    pub ang_vel_z: Option<f32>,
}

impl TimeSeriesCarData {
    pub fn from(
        actor: &Actor,
        attributes: &HashMap<String, boxcars::Attribute>,
        replay_version: i32,
    ) -> Self {
        let mut throttle = None;
        let mut steer = None;
        let mut handbrake = None;

        if let Some(Attribute::Byte(_throttle)) =
            attributes.get("TAGame.Vehicle_TA:ReplicatedThrottle")
        {
            throttle = Some(*_throttle);
        }
        if let Some(Attribute::Byte(_steer)) = attributes.get("TAGame.Vehicle_TA:ReplicatedSteer") {
            steer = Some(*_steer);
        }
        if let Some(Attribute::Byte(_handbrake)) =
            attributes.get("TAGame.Vehicle_TA:bReplicatedHandbrake")
        {
            handbrake = Some(*_handbrake);
        }

        let rigid_body_data = RigidBodyData::from(actor, attributes, replay_version);

        TimeSeriesCarData {
            throttle,
            steer,
            handbrake,
            is_sleeping: rigid_body_data.is_sleeping,
            pos_x: rigid_body_data.pos_x,
            pos_y: rigid_body_data.pos_y,
            pos_z: rigid_body_data.pos_z,
            vel_x: rigid_body_data.vel_x,
            vel_y: rigid_body_data.vel_y,
            vel_z: rigid_body_data.vel_z,
            quat_w: rigid_body_data.quat_w,
            quat_x: rigid_body_data.quat_x,
            quat_y: rigid_body_data.quat_y,
            quat_z: rigid_body_data.quat_z,
            ang_vel_x: rigid_body_data.ang_vel_x,
            ang_vel_y: rigid_body_data.ang_vel_y,
            ang_vel_z: rigid_body_data.ang_vel_z,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoData {
    pub frame_number: usize,
    pub attacker_wrapped_unique_id: WrappedUniqueId,
    pub victim_wrapped_unique_id: WrappedUniqueId,
}

impl DemoData {
    pub fn from(
        demolish: &std::boxed::Box<Demolish>,
        frame_number: usize,
        car_ids_to_player_ids: &HashMap<ActorId, ActorId>,
        players_wrapped_unique_id: &HashMap<ActorId, WrappedUniqueId>,
    ) -> Option<Self> {
        if !demolish.attacker_flag {
            // Attacker flag can be false and demolish.attacker == ActorId(-1).
            // I assume this is not a player-induced demolish. Could be goal explosion or goal reset or change team.
            return None;
        } else if demolish.attacker.0 == -1 {
            error!("Demo on frame {} where attacker is -1.", frame_number);
            return None;
        }
        Some(Self {
            frame_number,
            attacker_wrapped_unique_id: players_wrapped_unique_id
                .get(car_ids_to_player_ids.get(&demolish.attacker).unwrap())
                .unwrap()
                .clone(),
            victim_wrapped_unique_id: players_wrapped_unique_id
                .get(car_ids_to_player_ids.get(&demolish.victim).unwrap())
                .unwrap()
                .clone(),
        })
    }
}
