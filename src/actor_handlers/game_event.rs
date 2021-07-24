use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;

#[derive(Debug, Clone)]
pub struct GameEventHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for GameEventHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let attributes = actor.attributes.borrow();

        self.frame_parser
            .game_event_actor
            .replace(Some(attributes.clone())); // TODO: Optimise by avoiding premature clones? (i.e. Only clone final actor)

        let data = TimeSeriesGameEventData::from(actor);
        let mut game_event_data = self.frame_parser.time_series_game_event_data.borrow_mut();
        game_event_data.insert(frame_number, data);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesGameEventData {
    pub seconds_remaining: Option<i32>,
    pub replicated_game_state_time_remaining: Option<i32>,
    pub is_overtime: Option<bool>,
    pub ball_has_been_hit: Option<bool>,
}

impl TimeSeriesGameEventData {
    pub fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();

        let mut seconds_remaining = None;
        let mut replicated_game_state_time_remaining = None;
        let mut is_overtime = None;
        let mut ball_has_been_hit = None;

        if let Some(Attribute::Int(_seconds_remaining)) =
            attributes.get("TAGame.GameEvent_Soccar_TA:SecondsRemaining")
        {
            seconds_remaining = Some(*_seconds_remaining);
        }
        if let Some(Attribute::Int(_replicated_game_state_time_remaining)) =
            attributes.get("TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining")
        {
            replicated_game_state_time_remaining = Some(*_replicated_game_state_time_remaining);
        }
        if let Some(Attribute::Boolean(_is_overtime)) =
            attributes.get("TAGame.GameEvent_Soccar_TA:bOverTime")
        {
            is_overtime = Some(*_is_overtime);
        }
        if let Some(Attribute::Boolean(_ball_has_been_hit)) =
            attributes.get("TAGame.GameEvent_Soccar_TA:bBallHasBeenHit")
        {
            ball_has_been_hit = Some(*_ball_has_been_hit);
        }
        Self {
            seconds_remaining,
            replicated_game_state_time_remaining,
            is_overtime,
            ball_has_been_hit,
        }
    }
}
