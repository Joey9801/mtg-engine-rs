//! Observers that implement the base game rules to handle
//!   - The transition of game steps/phases
//!   - The passing of priority between players
//!   - Player input whe
//!
//! See sections 117 and 500 of the comprehensive rules

use core::{
    actions::{Action, ActionPayload, EngineAction, InputRequest},
    ids::{ObserverId, PlayerId},
    BaseObserver, PlayerInput,
};

use crate::{
    action::{AdvanceStep, MtgAction, MtgActionDowncast, PassPriority, SetPriority},
    game::Mtg,
    player_inputs::PriorityInput,
    steps::{BeginningStep, CombatStep, EndStep, GameStep, Step, SubStep},
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
fn next_step(game_state: &Mtg) -> GameStep {
    use BeginningStep::*;
    use CombatStep::*;
    use EndStep::*;
    use Step::*;

    // If the current step is in progress, the next thing to do is end it
    if game_state.step.substep.is_in_progress() {
        return GameStep {
            active_player: game_state.step.active_player,
            step: game_state.step.step,
            substep: SubStep::Ending,
        };
    }
    assert!(game_state.step.substep.is_ending());

    let next_step = match game_state.step.step {
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

    let next_active_player = if game_state.step.step == End(Cleanup) {
        game_state
            .turn_order
            .get(&game_state.step.active_player)
            .cloned()
            .expect("Don't know which player comes after the active player")
    } else {
        game_state.step.active_player
    };

    GameStep {
        active_player: next_active_player,
        step: next_step,
        substep: SubStep::InProgress,
    }
}

#[derive(Clone, Copy, Debug)]
enum ExpectedInput {
    /// The given player has priority, and is being asked what they would like to do
    Priority(PlayerId),
}

#[derive(Clone, Debug)]
pub struct StepsAndPriority {
    id: Option<ObserverId>,

    /// Tracks the number of consecutive priority passes
    ///
    /// Increments by one on each observed PassPriority
    /// Resets when any player performs an action that exercises their priority.
    passing_counter: usize,

    /// The next player that is going to receive priority
    next_priority: Option<PlayerId>,

    current_input_request: Option<ExpectedInput>,

    /// Actions to be emitted through the normal queuing mechanism after the EndInput action is
    /// observed.
    post_input_actions: Vec<ActionPayload<Mtg>>,
}

impl StepsAndPriority {
    pub fn new() -> Self {
        Self {
            id: None,
            passing_counter: 0,
            next_priority: None,
            current_input_request: None,
            post_input_actions: Vec::new(),
        }
    }

    fn handle_priority_input(
        &mut self,
        source: PlayerId,
        input: &PriorityInput,
        _game_state: &Mtg,
        emit_action: &mut dyn FnMut(ActionPayload<Mtg>),
    ) {
        match input {
            PriorityInput::PassPriority => {
                self.post_input_actions
                    .push(ActionPayload::DomainAction(
                        Box::new(PassPriority { player: source }) as Box<dyn MtgAction>,
                    ));
                emit_action(ActionPayload::EngineAction(EngineAction::EndInput));
            }
            PriorityInput::CastSpell => todo!(),
            PriorityInput::ActivateAbility => todo!(),
            PriorityInput::SpecialAction(_) => todo!(),
        }
    }
}

impl BaseObserver<Mtg> for StepsAndPriority {
    fn set_id(&mut self, id: ObserverId) {
        self.id = Some(id)
    }

    fn observe_action(
        &mut self,
        action: &Action<Mtg>,
        game_state: &Mtg,
        emit_action: &mut dyn FnMut(ActionPayload<Mtg>),
    ) {
        let self_id = self.id.expect("Don't have self id");

        match &action.payload {
            ActionPayload::EngineAction(EngineAction::NoActions) => {
                // The actions from whatever just happend have all calmed down now, and it is the
                // responsibility of this observer to kick something else off.
                // - If there is a player holding priority, we ask for their input
                // - If there is not a player holding priority, either:
                //   - Attempt to give the appropriate player priority
                //     - Except during the untap step, and (most) cleanup step(s)
                //   - Advance to the next step/substep

                if let Some(priority_player) = game_state.priority {
                    let input_req = InputRequest {
                        from_player: priority_player,
                        input_type: format!(
                            "Requesting priority input. Expecting MtgInput::PriorityInput(_)"
                        ),
                    };
                    emit_action(ActionPayload::EngineAction(EngineAction::RequestInput(
                        input_req.clone(),
                    )));
                    self.current_input_request = Some(ExpectedInput::Priority(priority_player));
                } else {
                    if game_state.step.substep == SubStep::Ending {
                        // There are no more things happening at the end of the current step, it is
                        // time to progress to the next step
                        let next_step = next_step(game_state);
                        let action = Box::new(AdvanceStep {
                            new_step: next_step.step,
                            new_substep: next_step.substep,
                            new_active_player: next_step.active_player,
                        }) as Box<dyn MtgAction>;
                        emit_action(ActionPayload::DomainAction(action));
                    } else {
                        // There should be a player ready to receive priority
                        let set_prio_action = Box::new(SetPriority {
                            new_priority: self
                                .next_priority
                                .expect("Don't know who should recieve priority next"),
                        }) as Box<dyn MtgAction>;
                        emit_action(ActionPayload::DomainAction(set_prio_action));
                    }
                }
            }
            ActionPayload::EngineAction(EngineAction::EndInput) if action.source == self_id => {
                for action in self.post_input_actions.drain(..) {
                    emit_action(action);
                }
            }
            ActionPayload::DomainAction(da) if da.is::<PassPriority>() => {
                let action = da.as_t::<PassPriority>().unwrap();
                self.passing_counter += 1;
                if self.passing_counter == game_state.players.len() {
                    // All players have passed priority in succession

                    // Whatever happens here, the passing counter is reset.
                    self.passing_counter = 0;

                    if game_state.stack().len() > 0 {
                        // There is something on the stack to resolve. Resolve that thing and grant
                        // the active player priority.
                        let resolve_action = game_state
                            .stack()
                            .top()
                            .unwrap()
                            .resolve_action
                            .clone()
                            .expect("Top of stack has no resolve action");

                        emit_action(ActionPayload::DomainAction(resolve_action));
                        self.next_priority = Some(game_state.step.active_player);
                    } else {
                        // There is nothing on the stack to resolve. Begin ending this step.
                        let advance_step_ending = Box::new(AdvanceStep {
                            new_step: game_state.step.step,
                            new_substep: SubStep::Ending,
                            new_active_player: game_state.step.active_player,
                        }) as Box<dyn MtgAction>;
                        emit_action(ActionPayload::DomainAction(advance_step_ending));

                        // This should be set to some value when the next step is seen starting
                        // Don't set it right away, as the active player on the next step could be
                        // different from what is expected here, eg if there is some extra turn
                        // effect.
                        self.next_priority = None;
                    }
                } else {
                    // Not all players have passed in succession yet, so work out which player
                    // should receive priority next. Don't actually emit a SetPriority action just
                    // yet though, instead wait for the next appropriate NoActions event.
                    let next_priority = game_state
                        .turn_order
                        .get(&action.player)
                        .cloned()
                        .expect("Don't know which player comes next in the turn order");
                    self.next_priority = Some(next_priority);
                }
            }
            ActionPayload::DomainAction(da) if da.is::<AdvanceStep>() => {
                let advance_step_action = da.as_t::<AdvanceStep>().unwrap();
                if advance_step_action.new_substep == SubStep::InProgress {
                    self.next_priority = Some(advance_step_action.new_active_player);
                }
            }
            _ => (),
        }
    }

    fn consume_input(
        &mut self,
        input: &PlayerInput<Mtg>,
        game_state: &Mtg,
        emit_action: &mut dyn FnMut(ActionPayload<Mtg>),
    ) {
        let expected = self
            .current_input_request
            .expect("Received input when not expecting one");

        match expected {
            ExpectedInput::Priority(p) => {
                // The engine should have already validated that the input came from the correct player
                assert_eq!(p, input.source);

                // TODO: don't panic when the wrong input is provided
                let prio_input = input
                    .payload
                    .as_domain_input()
                    .expect("Expected a domain input")
                    .as_priority_input()
                    .expect("Expected a priority input");

                self.handle_priority_input(input.source, prio_input, game_state, emit_action);
            }
        }
    }
}
