//! Observers that implement the base game rules for state-based actions
//!
//! See section 704 of the comprehensive rules

use core::{
    actions::{Action, ActionPayload},
    BaseObserver,
};

use crate::{
    action::{CompositeAction, MtgAction, MtgActionDowncast, SetPriority},
    game::Mtg,
};

#[derive(Debug, Clone)]
pub struct StateBasedActions {}

impl StateBasedActions {
    fn generate_actions(&self, _game_state: &Mtg) -> Option<CompositeAction> {
        // TODO: actually form a list of state based actions to take
        println!("Checking for state-based actions");
        None
    }
}

impl BaseObserver<Mtg> for StateBasedActions {
    fn propose_replacement(
        &self,
        action: &Action<Mtg>,
        game_state: &Mtg,
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
