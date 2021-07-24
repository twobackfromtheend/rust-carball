use crate::actor_handlers::{
    ActorHandler, BallHandler, BlueTeamHandler, BoostHandler, BoostPickupHandler, CarHandler,
    GameEventHandler, GameInfoHandler, OrangeTeamHandler, PlayerHandler,
};
use crate::frame_parser::FrameParser;

#[derive(Debug, Clone)]
pub struct ActorHandlerFactory<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandlerFactory<'a> {
    pub fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    pub fn get_handler(&'a self, object_name: &str) -> Option<Box<dyn ActorHandler + 'a>> {
        if object_name.ends_with(":GameReplicationInfoArchetype") {
            Some(Box::new(GameInfoHandler::new(self.frame_parser)))
        } else if object_name.starts_with("Archetypes.GameEvent.GameEvent") {
            Some(Box::new(GameEventHandler::new(self.frame_parser)))
        } else if object_name == "Archetypes.Teams.Team0" {
            Some(Box::new(BlueTeamHandler::new(self.frame_parser)))
        } else if object_name == "Archetypes.Teams.Team1" {
            Some(Box::new(OrangeTeamHandler::new(self.frame_parser)))
        } else if object_name.starts_with("Archetypes.Ball.") {
            Some(Box::new(BallHandler::new(self.frame_parser)))
        } else if object_name == "TAGame.Default__PRI_TA" {
            Some(Box::new(PlayerHandler::new(self.frame_parser)))
        } else if object_name == "Archetypes.Car.Car_Default" {
            Some(Box::new(CarHandler::new(self.frame_parser)))
        } else if object_name == "Archetypes.CarComponents.CarComponent_Boost" {
            Some(Box::new(BoostHandler::new(self.frame_parser)))
        } else if object_name.contains("TheWorld:PersistentLevel.VehiclePickup_Boost_TA") {
            // E.g. UtopiaStadium_Dusk_P.TheWorld:PersistentLevel.VehiclePickup_Boost_TA_6
            Some(Box::new(BoostPickupHandler::new(self.frame_parser)))
        } else {
            None
        }
    }
}
