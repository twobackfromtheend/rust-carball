use crate::actor_handlers::euler_from_quat;
use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::attributes::Demolish;
use boxcars::Attribute;
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
            // Assign car-player links
            self.frame_parser
                .car_ids_to_player_ids
                .borrow_mut()
                .insert(car_actor_id, player_actor_id);

            // Add time-series car data
            let car_data = TimeSeriesCarData::from(actor, &attributes); // Passed here as borrowed as mut above.
            let mut players_data = self.frame_parser.players_time_series_car_data.borrow_mut();
            match players_data.get_mut(&player_actor_id) {
                Some(player_data) => {
                    player_data.insert(frame_number, car_data);
                }
                None => {
                    let mut player_data =
                        HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                    player_data.insert(frame_number, car_data);
                    players_data.insert(player_actor_id, player_data);
                }
            }

            // Add demos
            if let Some(Attribute::Demolish(demolish)) =
                attributes.get("TAGame.Car_TA:ReplicatedDemolish")
            {
                let mut demos_data = self.frame_parser.demos_data.borrow_mut();
                demos_data.push(DemoData::from(demolish, frame_number));
                attributes.remove("TAGame.Car_TA:ReplicatedDemolish");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesCarData {
    pub is_sleeping: Option<bool>,
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub pos_z: Option<f32>,
    pub vel_x: Option<f32>,
    pub vel_y: Option<f32>,
    pub vel_z: Option<f32>,
    pub rot_pitch: Option<f32>,
    pub rot_yaw: Option<f32>,
    pub rot_roll: Option<f32>,
    pub ang_vel_x: Option<f32>,
    pub ang_vel_y: Option<f32>,
    pub ang_vel_z: Option<f32>,
    pub throttle: Option<u8>,
    pub steer: Option<u8>,
    pub handbrake: Option<u8>,
}

impl TimeSeriesCarData {
    pub fn from(actor: &Actor, attributes: &HashMap<String, boxcars::Attribute>) -> Self {
        let initial_location = actor
            .new_actor
            .initial_trajectory
            .location
            .expect("Car actor has no initial location.");

        let mut is_sleeping = None;
        let mut pos_x = Some(initial_location.x as f32);
        let mut pos_y = Some(initial_location.y as f32);
        let mut pos_z = Some(initial_location.z as f32);
        let mut vel_x = None;
        let mut vel_y = None;
        let mut vel_z = None;
        // TODO: Find out how initial_rotation should be used.
        // let initial_rotation = actor
        //     .new_actor
        //     .initial_trajectory
        //     .rotation
        //     .expect("Car actor has no initial rotation.");
        // let mut rot_pitch = initial_rotation.pitch.map(|rot| rot as f32);
        // let mut rot_yaw = initial_rotation.yaw.map(|rot| rot as f32);
        // let mut rot_roll = initial_rotation.roll.map(|rot| rot as f32);
        let mut rot_pitch = None;
        let mut rot_yaw = None;
        let mut rot_roll = None;
        let mut ang_vel_x = None;
        let mut ang_vel_y = None;
        let mut ang_vel_z = None;
        let mut throttle = None;
        let mut steer = None;
        let mut handbrake = None;

        if let Some(Attribute::RigidBody(rb_state)) =
            attributes.get("TAGame.RBActor_TA:ReplicatedRBState")
        {
            is_sleeping = Some(rb_state.sleeping);

            let location = rb_state.location;
            pos_x = Some(location.x);
            pos_y = Some(location.y);
            pos_z = Some(location.z);

            if let Some(linear_velocity) = rb_state.linear_velocity {
                if let Some(angular_velocity) = rb_state.angular_velocity {
                    vel_x = Some(linear_velocity.x);
                    vel_y = Some(linear_velocity.y);
                    vel_z = Some(linear_velocity.z);

                    let eulers = euler_from_quat(rb_state.rotation);
                    rot_pitch = Some(eulers.0);
                    rot_yaw = Some(eulers.1);
                    rot_roll = Some(eulers.2);

                    // Dividing by 100 to result in radians/s
                    ang_vel_x = Some(angular_velocity.x / 100.0);
                    ang_vel_y = Some(angular_velocity.y / 100.0);
                    ang_vel_z = Some(angular_velocity.z / 100.0);
                }
            }
        }

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
        TimeSeriesCarData {
            is_sleeping,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
            rot_pitch,
            rot_yaw,
            rot_roll,
            ang_vel_x,
            ang_vel_y,
            ang_vel_z,
            throttle,
            steer,
            handbrake,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoData {
    pub frame_number: usize,
    pub attacker_actor_id: boxcars::ActorId,
    pub victim_actor_id: boxcars::ActorId,
}

impl DemoData {
    pub fn from(demolish: &std::boxed::Box<Demolish>, frame_number: usize) -> Self {
        Self {
            frame_number,
            attacker_actor_id: demolish.attacker,
            victim_actor_id: demolish.victim,
        }
    }
}
