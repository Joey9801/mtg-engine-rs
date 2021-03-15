//! Observers that implement the base game rules for state-based actions
//!
//! See section 704 of the comprehensive rules

use crate::{
    actions::{
        mtg_action::{self, CompositeAction, MtgAction, MtgActionDowncast, SetPriority},
        Action, ActionPayload,
    },
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

    fn propose_replacement(
        &self,
        action: &Action,
        game_state: &GameState,
    ) -> Option<Box<dyn MtgAction>> {
        if let ActionPayload::DomainAction(a) = &action.payload {
            if a.is::<SetPriority>() {
                self.generate_actions(game_state)
                    .map(|composite| Box::new(composite) as Box<dyn MtgAction>)
            } else {
                None
            }
        } else {
            None
        }
    }
}
