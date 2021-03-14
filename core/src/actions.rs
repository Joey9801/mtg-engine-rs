use std::rc::Rc;

use crate::{
    game::{GameState, GameTimestamp},
    ids::ActionId,
    steps::{GameStep, Step, SubStep},
    zone::ZoneLocation,
    Controller, ObjectReference, ObserverId, PlayerId,
};

#[derive(Clone, Debug)]
pub struct CompositeAction {
    pub tag: &'static str,
    pub components: Vec<MtgAction>,
}

// TODO: This enum will certainly become unweildy. Replace it with a trait, and use a trait object as
// the DomainAction type.
// Can still serialize trait objects with https://github.com/dtolnay/typetag
#[derive(Clone, Debug)]
pub enum MtgAction {
    /// Advances the current GameStep to the given step/subset/player
    AdvanceStep(Step, SubStep, PlayerId),

    /// Move the given object to the given zone/location within that zone
    ChangeObjectZone(ObjectReference, ZoneLocation),

    /// The player that current has priority yields it
    ///
    /// Immediately after this action, no player has priority
    PassPriority,

    /// The given player recieves priority, or priority is cleared if the
    SetPriority(PlayerId),

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

impl MtgAction {
    pub fn apply(&self, game_state: &mut GameState) {
        match self {
            MtgAction::CompositeAction(c) => {
                for component in &c.components {
                    component.apply(game_state);
                }
            }
            MtgAction::SetPriority(player) => {
                game_state.priority = Some(*player);
            }
            MtgAction::PassPriority => {
                game_state.priority = None;
            }
            MtgAction::AdvanceStep(step, substep, active_player) => {
                game_state.step = GameStep {
                    active_player: *active_player,
                    step: *step,
                    substep: *substep,
                }
            }
            MtgAction::ChangeObjectZone(obj_ref, new_zone) => {
                let obj = match *obj_ref {
                    ObjectReference::Concrete(concrete_obj) => game_state
                        .zones
                        .get_mut(&concrete_obj.zone)
                        .expect("Failed to find zone in game state")
                        .remove(concrete_obj.object),
                    ObjectReference::Abstract(zone_loc) => {
                        let zone = game_state
                            .zones
                            .get_mut(&zone_loc.zone)
                            .expect("Failed to find zone in game state");

                        zone.resolve_abstract_zone_location(zone_loc.loc)
                            .map(|oid| zone.remove(oid))
                            .flatten()
                    }
                };

                if let Some(obj) = obj {
                    game_state
                        .zones
                        .get_mut(&new_zone.zone)
                        .expect("Failed to find zone in game state")
                        .insert(obj, new_zone.loc)
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputRequest {
    /// Input is being requested from this player
    pub from_player: PlayerId,

    /// Some token so that the player knows what input is being requested from them
    ///
    /// TODO: Could this be replaced with an enum/something more structured?
    /// A presentation layer on top of this engine would probably want to present specialized UI
    /// elements for each type of input
    pub input_type: String,
}

#[derive(Clone, Debug)]
pub enum EngineAction {
    /// Dummy action emitted by the game each time it is ticked with no actions in any queue
    ///
    /// The execution of this action has no effect on any game state
    NoActions,

    /// Starts an input session, with all inputs being directed toward the observer that created
    /// this action
    RequestInput(InputRequest),

    /// Ends the current input session
    EndInput,

    /// Picks the given action for the current round of replacement resolution
    ///
    /// Only valid when the action queue is part way through resolving some ambiguous replacements
    /// The action referenced must be one of the candidate replacements
    PickReplacement(ActionId),

    /// Picks the given action as the first one from the staging set that should be executed
    PickNextAction(ActionId)
}

#[derive(Clone, Debug)]
pub enum ActionPayload {
    /// An action that represents some core engine activity unrelated to any domain state
    EngineAction(EngineAction),

    /// An action that represents an atomic modification to the domain state
    DomainAction(MtgAction),
}

#[derive(Clone, Debug)]
pub struct Action {
    /// The actual sub-operation that this action will perform
    pub payload: ActionPayload,

    /// The player controlling this action, if any
    ///
    /// Necessary as part of ordering simultaneous actions.
    /// Will be None if the action originated from the game itself
    pub controller: Controller,

    /// The observer that added this action to the queue
    pub source: ObserverId,

    /// Globally unique ID for this action
    ///
    /// Each candidate replacement effect will have its own new ID, such that it is possible to
    /// refer to a particular candidate replacement effect. After a replacement is committed, the
    /// resolved action will retain this new ID.
    pub id: ActionId,

    /// The GameTimestamp when this action was first emitted
    ///
    /// Even if replacement effects modify this action at a later time, this timestamp will
    /// persist.
    pub generated_at: GameTimestamp,

    /// If this action was the result of a replacement effect, the original action that it replaced
    pub original: Option<Rc<Action>>,
}

impl Action {
    pub fn root_source(&self) -> ObserverId {
        match &self.original {
            Some(a) => a.root_source(),
            None => self.source,
        }
    }
}
