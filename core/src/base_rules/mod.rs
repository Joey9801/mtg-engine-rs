use crate::game::Game;

pub mod state_actions;

use state_actions::StateBasedActions;

pub fn attach(game: &mut Game) {
    game.attach_observer(Box::new(StateBasedActions::new()));
}