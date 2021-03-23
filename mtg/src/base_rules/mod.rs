use core::game::Game;

pub mod combat;
pub mod progression;
pub mod state_actions;

use crate::game::Mtg;
use combat::CombatManager;
use progression::StepsAndPriority;
use state_actions::StateBasedActions;

pub fn attach(game: &mut Game<Mtg>) {
    game.attach_observer(Box::new(StateBasedActions {}));
    game.attach_observer(Box::new(StepsAndPriority::new()));
    game.attach_observer(Box::new(CombatManager::new()));
}
