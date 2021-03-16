use mtg_engine_core::game::Game;

pub mod progression;
pub mod state_actions;

use crate::MtgGameState;
use progression::StepsAndPriority;
use state_actions::StateBasedActions;

pub fn attach(game: &mut Game<MtgGameState>) {
    game.attach_observer(Box::new(StateBasedActions {}));
    game.attach_observer(Box::new(StepsAndPriority::new()));
}
