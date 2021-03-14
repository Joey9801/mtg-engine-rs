pub mod actions;
pub mod base_rules;
pub mod game;
pub mod ids;
pub mod steps;
pub mod zone;

use actions::{Action, ActionPayload, MtgAction};
use game::GameState;
use zone::ZoneLocation;

use ids::{ObjectId, ObserverId, PlayerId, ZoneId};

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub life_total: i32,
    pub library: ZoneId,
    pub hand: ZoneId,
    pub graveyard: ZoneId,
}

/// A game object that can exist in a zone
#[derive(Clone, Debug)]
pub struct Object {
    pub id: ObjectId,
    pub owner: PlayerId,
    pub controller: PlayerId,

    /// The action to be executed if/when this object is resolved from the top of the stack.
    ///
    /// Only relevant for objects on the stack.
    /// This action will be added to the staging set and subject to replacement effects just like
    /// any other.
    pub resolve_action: Option<MtgAction>,
}

#[derive(Clone, Copy, Debug)]
pub enum Controller {
    Game,
    Player(PlayerId),
}

#[derive(Clone, Copy, Debug)]
pub struct ConcreteObject {
    pub zone: ZoneId,
    pub object: ObjectId,
}

#[derive(Clone, Debug)]
pub enum ObjectReference {
    Concrete(ConcreteObject),
    Abstract(ZoneLocation),
}

#[derive(Clone, Copy, Debug)]
pub enum MtgValue {
    Number(i32),
    Flag(bool),
    Object(ObjectId),
    Player(PlayerId),
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerInputPayload {
    Data(MtgValue),
    FinishedInput,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerInput {
    pub source: PlayerId,
    pub payload: PlayerInputPayload,
}

pub trait BaseObserver: std::fmt::Debug {
    /// Who owns this effect.
    fn controller(&self) -> Controller {
        Controller::Game
    }

    /// Called once when the game allocates this observer its globally unique ID
    ///
    /// Any actions emitted by a given observer will have that observers ID attached to them by the
    /// game. This property can be used to safely implement internal state machines without
    /// accidentally reacting to actions emitted by different observers.
    fn set_id(&mut self, _id: ObserverId) {}

    /// If this observer is no longer relevant, returning false from this method will cause it to
    /// be cleaned up.
    fn alive(&self, _game: &GameState) -> bool {
        true
    }

    /// An opportunity for this observer to mutate an action before it gets queued for application.
    ///
    /// Replacement actions proposed in this manner are not gauranteed to be applied. In particular
    /// if there are multiple competing replacement actions, either one or zero of those
    /// replacements may be picked based on a combination of game rules and player choice.
    ///
    /// Only domain actions may be modified
    fn propose_replacement(&self, _action: &Action, _game: &GameState) -> Option<MtgAction> {
        None
    }

    /// The given action has just been applied to the game state, this is this effect's chance to
    /// react to it.
    ///
    /// If this effect would like to perform another action in reaction to the observed one, it
    /// should add it to the game's staging action set.
    fn observe_action(
        &mut self,
        _action: &Action,
        _game_state: &GameState,
        _emit_action: &mut dyn FnMut(ActionPayload),
    ) {
    }

    /// If this observer has emitted a RequestInput action, this method will be called with each
    /// input the player makes
    ///
    /// Actions emitted from this method will be applied to the game immediately, bypassing the
    /// regular action queue. The game will continue requesting input from the player until the
    /// EndInput action is emitted.
    ///
    /// TODO: Add a mechanism for the observer to indicate that the given input was invalid
    /// (perhaps just returning a Result<T, E> from this method)
    fn consume_input(
        &mut self,
        _input: &PlayerInput,
        _game_state: &GameState,
        _emit_action: &mut dyn FnMut(ActionPayload),
    ) {
        panic!("Input being passed to an observer that has no consume_input implementation")
    }
}

pub trait Observer: BaseObserver {
    fn clone_box(&self) -> Box<dyn Observer>;
}

impl<T: 'static + BaseObserver + Clone> Observer for T {
    fn clone_box(&self) -> Box<dyn Observer> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Observer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
