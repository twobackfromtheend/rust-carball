use crate::frame_parser::{Actor, FrameParser};

pub trait ActorHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self
    where
        Self: Sized;

    fn update(&mut self, actor: &Actor, frame_number: usize, time: f32, delta: f32);
}
