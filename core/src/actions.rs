use std::rc::Rc;

use crate::{game::GameTimestamp, ids::ActionId, GameDomain, Observer, ObserverId, PlayerId};

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
pub enum EngineAction<TGame: GameDomain> {
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
    PickNextAction(ActionId),

    /// Attaches the given new observer to the engine
    ///
    /// The newly attached observer will be notified of its own ID via the `set_id` method before
    /// it observes any actions.
    /// The first action the newly attached observer will observe will be the action that attached
    /// it to the game.
    AttachObserver(Box<dyn Observer<TGame>>),
}

#[derive(Clone, Debug)]
pub enum ActionPayload<TGame: GameDomain> {
    /// An action that represents some core engine activity unrelated to any domain state
    EngineAction(EngineAction<TGame>),

    /// An action that represents an atomic modification to the domain state
    DomainAction(TGame::Action),

    Composite(Vec<Action<TGame>>),
}

#[derive(Clone, Debug)]
pub struct Action<TGame: GameDomain> {
    /// The actual sub-operation that this action will perform
    pub payload: ActionPayload<TGame>,

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
    pub original: Option<Rc<Action<TGame>>>,
}

impl<TGame: GameDomain> Action<TGame> {
    pub fn root_source(&self) -> ObserverId {
        match &self.original {
            Some(a) => a.root_source(),
            None => self.source,
        }
    }
}
