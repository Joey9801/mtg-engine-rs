//! Observers that implement the base game rules for state-based actions
//!
//! See section 704 of the comprehensive rules

use crate::{
    actions::{Action, BaseAction, CompositeAction},
    game::Game,
    ids::{ObserverId, PlayerId},
    BaseObserver, Controller,
};

#[derive(Debug, Clone)]
pub struct StateBasedActions {
    id: Option<ObserverId>,

    /// The player who was about to recieve priority before this the current
    /// round of state-based actions started.
    pending_priority: Option<PlayerId>,
}

impl StateBasedActions {
    pub fn new() -> Self {
        Self {
            id: None,
            pending_priority: None,
        }
    }

    fn generate_actions(&self, game: &Game) -> Option<CompositeAction> {
        // TODO: actually form a list of state based actions to take
        println!("Checking for state-based actions");
        None
    }
}

impl BaseObserver for StateBasedActions {
    fn controller(&self) -> Controller {
        Controller::Game
    }
    fn set_id(&mut self, id: ObserverId) {
        self.id = Some(id);
    }

    fn propose_replacement(&mut self, action: &Action, game: &Game) -> Option<BaseAction> {
        if let BaseAction::SetPriority(player) = action.base_action {
            let sba = self.generate_actions(game);
            if sba.is_some() {
                self.pending_priority = Some(player);
            }

            sba.map(|composite| BaseAction::CompositeAction(composite))
        } else {
            None
        }
    }

    fn observe_action(
        &mut self,
        action: &Action,
        _game: &Game,
        emit_action: &mut dyn FnMut(BaseAction),
    ) {
        // If there are no actions being executed
        if let BaseAction::NoActions = action.base_action {
            // And we're currently holding up priority for state-based actions
            if let Some(player) = self.pending_priority {
                // Attempt to give that player priority again
                // The replacement effect above wont let this through if another round of state-based actions is required
                emit_action(BaseAction::SetPriority(player));
                self.pending_priority = None;
            }
        }
    }
}
