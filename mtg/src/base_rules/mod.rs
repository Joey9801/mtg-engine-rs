use core::game::Game;

pub mod progression;
pub mod state_actions;
pub mod combat;

use crate::game::Mtg;
use progression::StepsAndPriority;
use state_actions::StateBasedActions;
use combat::CombatManager;

pub fn attach(game: &mut Game<Mtg>) {
    game.attach_observer(Box::new(StateBasedActions {}));
    game.attach_observer(Box::new(StepsAndPriority::new()));
    game.attach_observer(Box::new(CombatManager::new()));
}
