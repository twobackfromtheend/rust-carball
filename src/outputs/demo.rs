use crate::actor_handlers::{DemoData, WrappedUniqueId};
use crate::frame_parser::FrameParser;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Demo {
    frame_number: i32,
    attacker_unique_id: WrappedUniqueId,
    victim_unique_id: WrappedUniqueId,
}

impl Demo {
    pub fn from_frame_parser(frame_parser: &FrameParser) -> Vec<Self> {
        let demos_data = frame_parser.demos_data.borrow();
        demos_data.iter().map(Demo::from).collect()
    }

    pub fn from(demo_data: &DemoData) -> Self {
        Self {
            frame_number: demo_data.frame_number as i32,
            attacker_unique_id: demo_data.attacker_wrapped_unique_id.clone(),
            victim_unique_id: demo_data.victim_wrapped_unique_id.clone(),
        }
    }
}
