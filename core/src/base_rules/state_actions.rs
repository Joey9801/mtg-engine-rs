//! Observers that implement the base game rules for state-based actions
//!
//! See section 704 of the comprehensive rules

use crate::{
    actions::{Action, ActionPayload, CompositeAction, MtgAction},
    game::GameState,
    BaseObserver, Controller,
};

#[derive(Debug, Clone)]
pub struct StateBasedActions {}

impl StateBasedActions {
    fn generate_actions(&self, _game_state: &GameState) -> Option<CompositeAction> {
        // TODO: actually form a list of state based actions to take
        println!("Checking for state-based actions");
        None
    }
}

impl BaseObserver for StateBasedActions {
    fn controller(&self) -> Controller {
        Controller::Game
    }

    fn propose_replacement(&self, action: &Action, game_state: &GameState) -> Option<MtgAction> {
        if let ActionPayload::DomainAction(MtgAction::SetPriority(_)) = action.payload {
            self.generate_actions(game_state)
                .map(|composite| MtgAction::CompositeAction(composite))
        } else {
            None
        }
    }
}
