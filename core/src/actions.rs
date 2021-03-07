use crate::{Controller, ObjectReference, ObserverId, PlayerId, game::Game, steps::{Step, SubStep}, zone::ZoneLocation};

#[derive(Clone, Debug)]
pub struct CompositeAction {
    pub tag: &'static str,
    pub components: Vec<BaseAction>,
}

#[derive(Clone, Debug)]
pub enum BaseAction {
    /// Advances the current substep from InProgress -> Ending
    EndStep,

    /// Advances the current GameStep to the given { step/player, InProgress }
    AdvanceStep(Step, PlayerId),

    /// Move the given object to the given zone/location within that zone
    ChangeObjectZone(ObjectReference, ZoneLocation),

    /// The player that current has priority yields it
    ///
    /// Immediately after this action, no player has priority
    PassPriority,

    /// The given player recieves priority
    ///
    /// Should only be emitted after all state-based actions have finished resolving
    SetPriority(PlayerId),

    /// No-op action to announce that state based actions have finished
    FinishStateBasedActions,

    /// Some higher level actions are formed of one or more other base actions and a tag
    ///
    /// An example of a higher level action is drawing a card. Drawing a single card has the components:
    ///     - tag: "draw"
    ///     - action: ChangeObjectZone; top of deck -> hand
    ///
    /// This form gives observers that explicitly care about the higher level action something to look for
    /// It also simplifies observers that only care about the lower level actions (eg "whenever a card is put into a your hand from anywhere")
    CompositeAction(CompositeAction),
}

impl BaseAction {
    pub fn apply(&self, game_state: &mut Game) {
        match self {
            BaseAction::EndStep => {
                debug_assert!(game_state.step.substep == SubStep::InProgress);
                game_state.step.substep = SubStep::Ending;
            }
            BaseAction::PassPriority => {
                debug_assert!(game_state.priority.is_some());
                game_state.priority = None;
            }
            BaseAction::SetPriority(player) => {
                debug_assert!(game_state.priority.is_none());
                game_state.priority = Some(*player);
            }
            // Just a marker, nothing to actually do
            BaseAction::FinishStateBasedActions => (),
            BaseAction::CompositeAction(c) => {
                for component in &c.components {
                    component.apply(game_state);
                }
            }
            _ => todo!()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Action {
    /// The actual sub-operation that this action will perform
    pub base_action: BaseAction,

    /// The player controlling this action, if any
    ///
    /// Necessary as part of ordering simultaneous actions.
    /// Will be None if the action originated from the game itself
    pub controller: Controller,
    
    /// The observer that added this action to the queue
    pub source: ObserverId,

    /// If this action was the result of a replacement effect, the original action that it replaced
    pub original: Option<Box<Action>>,
}
