use crate::frame_parser::{Actor, FrameParser};
use std::slice::Iter;

pub trait ActorHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self
    where
        Self: Sized;

    fn update(&mut self, actor: &Actor, frame_number: usize, time: f32, delta: f32);

    fn priority(&self) -> ActorHandlerPriority {
        ActorHandlerPriority::Standard
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActorHandlerPriority {
    First,
    Standard,
}

impl ActorHandlerPriority {
    pub fn iterator() -> Iter<'static, Self> {
        static ACTOR_HANDLER_PRIORITIES: [ActorHandlerPriority; 2] =
            [ActorHandlerPriority::First, ActorHandlerPriority::Standard];
        ACTOR_HANDLER_PRIORITIES.iter()
    }
}
