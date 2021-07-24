use crate::actor_handlers::euler_from_quat;
use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;

#[derive(Debug, Clone)]
pub struct BallHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for BallHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        // Add time-series ball data
        let mut ball_data = self.frame_parser.time_series_ball_data.borrow_mut();
        let _ball_data = TimeSeriesBallData::from(actor);
        ball_data.insert(frame_number, _ball_data);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesBallData {
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
    pub hit_team_num: Option<u8>,
}

impl TimeSeriesBallData {
    pub fn from(actor: &Actor) -> Self {
        let initial_location = actor
            .new_actor
            .initial_trajectory
            .location
            .expect("Car actor has no initial location.");
        let initial_rotation = actor
            .new_actor
            .initial_trajectory
            .location
            .expect("Car actor has no initial rotation.");

        let mut is_sleeping = None;
        let mut pos_x = Some(initial_location.x as f32);
        let mut pos_y = Some(initial_location.y as f32);
        let mut pos_z = Some(initial_location.z as f32);
        let mut vel_x = None;
        let mut vel_y = None;
        let mut vel_z = None;
        let mut rot_pitch = Some(initial_rotation.x as f32);
        let mut rot_yaw = Some(initial_rotation.y as f32);
        let mut rot_roll = Some(initial_rotation.z as f32);
        let mut ang_vel_x = None;
        let mut ang_vel_y = None;
        let mut ang_vel_z = None;
        let mut hit_team_num = None;

        let attributes = actor.attributes.borrow();

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

        if let Some(Attribute::Byte(_hit_team_num)) = attributes.get("TAGame.Ball_TA:HitTeamNum") {
            hit_team_num = Some(*_hit_team_num);
        }
        TimeSeriesBallData {
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
            hit_team_num,
        }
    }
}
