use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};

#[derive(Debug, Clone)]
pub struct GameInfoHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for GameInfoHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, _frame_number: usize, _time: f32, _delta: f32) {
        let attributes = actor.attributes.borrow();

        self.frame_parser
            .game_info_actor
            .replace(Some(attributes.clone())); // TODO: Optimise by avoiding premature clones? (i.e. Only clone final actor)
    }
}
