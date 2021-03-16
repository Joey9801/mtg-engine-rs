//! Observers that implement the base game rules for state-based actions
//!
//! See section 704 of the comprehensive rules

use mtg_engine_core::{
    actions::{Action, ActionPayload},
    BaseObserver, Controller,
};

use crate::{
    mtg_action::{CompositeAction, MtgAction, MtgActionDowncast, SetPriority},
    MtgGameState,
};

#[derive(Debug, Clone)]
pub struct StateBasedActions {}

impl StateBasedActions {
    fn generate_actions(&self, _game_state: &MtgGameState) -> Option<CompositeAction> {
        // TODO: actually form a list of state based actions to take
        println!("Checking for state-based actions");
        None
    }
}

impl BaseObserver<MtgGameState> for StateBasedActions {
    fn controller(&self) -> Controller {
        Controller::Game
    }

    fn propose_replacement(
        &self,
        action: &Action<MtgGameState>,
        game_state: &MtgGameState,
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
