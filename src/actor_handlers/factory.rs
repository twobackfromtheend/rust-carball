use crate::actor_handlers::{
    ActorHandler, BallHandler, BlueTeamHandler, BoostHandler, CarHandler, GameEventHandler,
    GameInfoHandler, OrangeTeamHandler, PlayerHandler,
};
use crate::frame_parser::FrameParser;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ActorHandlerFactory<'a> {
    frame_parser: &'a FrameParser,
    object_id_to_handler_cache: RefCell<HashMap<boxcars::ObjectId, ActorKind>>,
}

impl<'a> ActorHandlerFactory<'a> {
    pub fn new(frame_parser: &'a FrameParser) -> Self {
        Self {
            frame_parser,
            object_id_to_handler_cache: RefCell::new(HashMap::default()),
        }
    }

    pub fn get_handler(
        &'a self,
        object_id: boxcars::ObjectId,
        replay_objects: &[String],
    ) -> Option<Box<dyn ActorHandler + 'a>> {
        let mut object_id_to_handler_cache = self.object_id_to_handler_cache.borrow_mut();
        let actor_kind = match object_id_to_handler_cache.get(&object_id) {
            Some(actor_kind) => *actor_kind,
            None => {
                let actor_kind = ActorKind::get_actor_kind(object_id, replay_objects);
                object_id_to_handler_cache.insert(object_id, actor_kind);
                actor_kind
            }
        };
        self.get_handler_for_actor_kind(actor_kind)
    }

    pub fn get_handler_for_actor_kind(
        &'a self,
        actor_kind: ActorKind,
    ) -> Option<Box<dyn ActorHandler + 'a>> {
        match actor_kind {
            ActorKind::GameInfo => Some(Box::new(GameInfoHandler::new(self.frame_parser))),
            ActorKind::GameEvent => Some(Box::new(GameEventHandler::new(self.frame_parser))),
            ActorKind::BlueTeam => Some(Box::new(BlueTeamHandler::new(self.frame_parser))),
            ActorKind::OrangeTeam => Some(Box::new(OrangeTeamHandler::new(self.frame_parser))),
            ActorKind::Ball => Some(Box::new(BallHandler::new(self.frame_parser))),
            ActorKind::Player => Some(Box::new(PlayerHandler::new(self.frame_parser))),
            ActorKind::Car => Some(Box::new(CarHandler::new(self.frame_parser))),
            ActorKind::Boost => Some(Box::new(BoostHandler::new(self.frame_parser))),
            ActorKind::NotHandled => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    GameInfo,
    GameEvent,
    BlueTeam,
    OrangeTeam,
    Ball,
    Player,
    Car,
    Boost,
    NotHandled,
}

impl ActorKind {
    pub fn get_actor_kind(object_id: boxcars::ObjectId, replay_objects: &[String]) -> Self {
        let object_name = &replay_objects[usize::from(object_id)];
        if object_name.ends_with(":GameReplicationInfoArchetype") {
            Self::GameInfo
        } else if object_name.starts_with("Archetypes.GameEvent.GameEvent") {
            Self::GameEvent
        } else if object_name == "Archetypes.Teams.Team0" {
            Self::BlueTeam
        } else if object_name == "Archetypes.Teams.Team1" {
            Self::OrangeTeam
        } else if object_name.starts_with("Archetypes.Ball.") {
            Self::Ball
        } else if object_name == "TAGame.Default__PRI_TA" {
            Self::Player
        } else if object_name == "Archetypes.Car.Car_Default" {
            Self::Car
        } else if object_name == "Archetypes.CarComponents.CarComponent_Boost" {
            Self::Boost
        } else {
            Self::NotHandled
        }
    }
}
