use crate::actor_handlers::{ActorHandler, RigidBodyData};
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
        let _ball_data = TimeSeriesBallData::from(actor, self.frame_parser.replay_version);
        ball_data.insert(frame_number, _ball_data);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesBallData {
    pub is_sleeping: Option<bool>,
    pub hit_team_num: Option<u8>,
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

impl TimeSeriesBallData {
    pub fn from(actor: &Actor, replay_version: i32) -> Self {
        let mut hit_team_num = None;
        let attributes = actor.attributes.borrow();
        if let Some(Attribute::Byte(_hit_team_num)) = attributes.get("TAGame.Ball_TA:HitTeamNum") {
            hit_team_num = Some(*_hit_team_num);
        }
        let rigid_body_data = RigidBodyData::from(actor, &attributes, replay_version);

        TimeSeriesBallData {
            hit_team_num,
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
