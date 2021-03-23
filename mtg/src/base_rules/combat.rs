use core::{
    actions::{Action, ActionPayload, EngineAction, InputRequest},
    ids::ObserverId,
    BaseObserver, PlayerInput,
};

use crate::{
    action::{AdvanceStep, MtgActionDowncast},
    game::Mtg,
    player_inputs::MtgInput,
    steps::{CombatStep, Step, SubStep},
};

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum ExpectedInput {
    /// Expect the player to nominate the next object to be an attacker, or that they have finished
    /// declaring attackers
    NextAttackerOrFinished,

    /// Expect the player to nominate what the most recently declared attacker is attacking
    NextAttackee,
}

#[derive(Clone, Debug)]
pub struct CombatManager {
    id: Option<ObserverId>,
    current_input_request: Option<ExpectedInput>,
}

impl CombatManager {
    pub fn new() -> Self {
        Self {
            id: None,
            current_input_request: None,
        }
    }
}

impl BaseObserver<Mtg> for CombatManager {
    fn set_id(&mut self, id: ObserverId) {
        self.id = Some(id)
    }

    fn alive(&self, _game: &Mtg) -> bool {
        true
    }

    fn observe_action(
        &mut self,
        action: &Action<Mtg>,
        game_state: &Mtg,
        emit_action: &mut dyn FnMut(ActionPayload<Mtg>),
    ) {
        match &action.payload {
            ActionPayload::DomainAction(da) => {
                if let Some(da) = da.as_t::<AdvanceStep>() {
                    if let Step::Combat(CombatStep::DeclareAttackers) = da.new_step {
                        if let SubStep::InProgress = da.new_substep {
                            // This is the beginning of the declare attackers step
                            self.current_input_request =
                                Some(ExpectedInput::NextAttackerOrFinished);
                            emit_action(ActionPayload::EngineAction(EngineAction::RequestInput(
                                InputRequest {
                                    from_player: game_state.step.active_player,
                                    input_type: format!(
                                        "{} to declare attackers",
                                        game_state.step.active_player
                                    ),
                                },
                            )))
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn consume_input(
        &mut self,
        input: &PlayerInput<Mtg>,
        _game_state: &Mtg,
        emit_action: &mut dyn FnMut(core::actions::ActionPayload<Mtg>),
    ) {
        let expected = self
            .current_input_request
            .expect("Received input when not expecting one");

        match expected {
            ExpectedInput::NextAttackerOrFinished => {
                let input = input
                    .payload
                    .as_domain_input()
                    .expect("Expected a domain input");

                match input {
                    MtgInput::Finished => {
                        emit_action(ActionPayload::EngineAction(EngineAction::EndInput));
                    }
                    MtgInput::ObjectId(_obj_id) => todo!("Implement declaring attackers"),
                    _ => panic!("Received bad input"),
                }
            }
            ExpectedInput::NextAttackee => {
                let input = input
                    .payload
                    .as_domain_input()
                    .expect("Expected a domain input");

                match input {
                    MtgInput::ObjectId(_obj_id) => {
                        todo!("Implement declaring object attack target")
                    }
                    MtgInput::PlayerId(_player_id) => {
                        todo!("Implement declaring player attack target")
                    }
                    _ => panic!("Received bad input"),
                }
            }
        }
    }
}
