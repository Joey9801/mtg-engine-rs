//! Observers that implement the base game rules to handle
//!   - The transition of game steps/phases
//!   - The passing of priority between players
//!   - Player input whe
//!
//! See sections 117 and 500 of the comprehensive rules

use mtg_engine_core::{
    actions::{Action, ActionPayload},
    ids::ObserverId,
    BaseObserver, Controller, PlayerInput,
};

use crate::{
    mtg_action::{MtgActionDowncast, PassPriority},
    steps::{BeginningStep, CombatStep, EndStep, Step},
    MtgGameState,
};

/// Does the given step normally involve a round of priority
fn step_has_priority(step: &Step) -> bool {
    match step {
        Step::Beginning(BeginningStep::Untap) => false,
        Step::End(EndStep::Cleanup) => false,
        _ => true,
    }
}

/// The next next step under the default ordering, and whether the active player should advance
fn default_next_step(step: &Step) -> (Step, bool) {
    use BeginningStep::*;
    use CombatStep::*;
    use EndStep::*;
    use Step::*;

    let next_step = match step {
        Beginning(Untap) => Beginning(Upkeep),
        Beginning(Upkeep) => Beginning(Draw),
        Beginning(Draw) => PreCombatMain,
        PreCombatMain => Combat(StartOfCombat),
        Combat(StartOfCombat) => Combat(DeclareAttackers),
        Combat(DeclareAttackers) => Combat(DeclareBlockers),
        Combat(DeclareBlockers) => Combat(CombatDamage),
        Combat(CombatDamage) => Combat(EndOfCombat),
        Combat(EndOfCombat) => PostCombatMain,
        PostCombatMain => End(EndOfTurn),
        End(EndOfTurn) => End(Cleanup),
        End(Cleanup) => Beginning(Untap),
        Starting(_) => panic!("default_next_step being used on special starting steps"),
    };

    let next_player = *step == End(Cleanup);

    (next_step, next_player)
}

enum NextState {
    /// The active player recieves priority
    FirstPriority,

    /// The next player in turn order receives priority
    NextPriority,

    /// The top of the stack is resolved
    ResolveStackItem,

    /// The next phase/step is started
    NextStep,
}

#[derive(Clone, Debug)]
pub struct StepsAndPriority {
    id: Option<ObserverId>,

    /// Tracks the number of consecutive priority passes
    ///
    /// Increments by one on each observed PassPriority
    /// Resets when any player performs an action that exercises their priority.
    passing_counter: usize,
}

impl StepsAndPriority {
    pub fn new() -> Self {
        Self {
            id: None,
            passing_counter: 0,
        }
    }

    fn next_state(&self, game: &MtgGameState) -> NextState {
        if step_has_priority(&game.step.step) {
            if self.passing_counter == game.players.len() {
                if game.stack().len() == 0 {
                    NextState::NextStep
                } else {
                    NextState::ResolveStackItem
                }
            } else {
                NextState::NextStep
            }
        } else {
            NextState::NextStep
        }
    }
}

impl BaseObserver<MtgGameState> for StepsAndPriority {
    fn controller(&self) -> Controller {
        Controller::Game
    }

    fn set_id(&mut self, id: ObserverId) {
        self.id = Some(id)
    }

    fn observe_action(
        &mut self,
        action: &Action<MtgGameState>,
        _game: &MtgGameState,
        _emit_action: &mut dyn FnMut(ActionPayload<MtgGameState>),
    ) {
        // NB: there aren't currently any actions that represent a player utilizing their priority
        match &action.payload {
            ActionPayload::DomainAction(a) if a.is::<PassPriority>() => self.passing_counter += 1,
            ActionPayload::DomainAction(_) => self.passing_counter = 0,
            _ => (),
        }
    }

    fn consume_input(
        &mut self,
        _input: &PlayerInput<MtgGameState>,
        _game_state: &MtgGameState,
        _emit_action: &mut dyn FnMut(ActionPayload<MtgGameState>),
    ) {
        panic!("Input being passed to an observer that has no consume_input implementation")
    }
}
