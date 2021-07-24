use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BlueTeamHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for BlueTeamHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, _frame_number: usize, _time: f32, _delta: f32) {
        let actor_id = actor.new_actor.actor_id;
        let attributes = actor.attributes.borrow();
        let mut teams_actor = self.frame_parser.teams_data.borrow_mut();
        teams_actor.insert(actor_id, TeamData::from(false, &attributes));
    }
}

#[derive(Debug, Clone)]
pub struct OrangeTeamHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for OrangeTeamHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, _frame_number: usize, _time: f32, _delta: f32) {
        let actor_id = actor.new_actor.actor_id;
        let attributes = actor.attributes.borrow();
        let mut teams_actor = self.frame_parser.teams_data.borrow_mut();
        teams_actor.insert(actor_id, TeamData::from(true, &attributes));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamData {
    pub is_orange: bool,
    pub score: i32,
}

impl TeamData {
    pub fn from(is_orange: bool, attributes: &HashMap<String, boxcars::Attribute>) -> Self {
        Self {
            is_orange,
            score: match attributes.get("Engine.TeamInfo:Score") {
                Some(Attribute::Int(score)) => *score,
                _ => 0,
            },
        }
    }
}
