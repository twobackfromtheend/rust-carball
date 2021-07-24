use crate::actor_handlers::TeamData;
use crate::frame_parser::FrameParser;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Team {
    score: i32,
    is_orange: bool,
}

impl Team {
    pub fn from_frame_parser(frame_parser: &FrameParser) -> Vec<Self> {
        let teams_data = frame_parser.teams_data.borrow();
        teams_data.values().map(Team::from).collect()
    }

    pub fn from(data: &TeamData) -> Self {
        Self {
            score: data.score,
            is_orange: data.is_orange,
        }
    }
}
