pub mod actions;
pub mod game;
pub mod ids;

use actions::{Action, ActionPayload};
use game::GameDomain;
use ids::{ActionId, ObserverId, PlayerId};

#[derive(Clone, Copy, Debug)]
pub enum Controller {
    Game,
    Player(PlayerId),
}

/// An input the player can give to be consumed by the engine itself
#[derive(Clone, Copy, Debug)]
pub enum EngineInput {
    /// Used for:
    /// - Picking a single candidate replacement effect when multiple could apply
    /// - Picking the next action to queue up for execution when the order is ambiguous
    ActionId(ActionId),
}

#[derive(Clone, Debug)]
pub enum PlayerInputPayload<TGame: GameDomain> {
    /// Inputs intended for the engine itselfj
    EngineInput(EngineInput),

    /// Domain specific inputs understood by game specific observers
    DomainInput(TGame::Input),
}

#[derive(Clone, Debug)]
pub struct PlayerInput<TGame: GameDomain> {
    pub source: PlayerId,
    pub payload: PlayerInputPayload<TGame>,
}

pub trait BaseObserver<TGame: GameDomain>: std::fmt::Debug {
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
    fn alive(&self, _game: &TGame) -> bool {
        true
    }

    /// An opportunity for this observer to mutate an action before it gets queued for application.
    ///
    /// Replacement actions proposed in this manner are not gauranteed to be applied. In particular
    /// if there are multiple competing replacement actions, either one or zero of those
    /// replacements may be picked based on a combination of game rules and player choice.
    ///
    /// Only domain actions may be modified
    fn propose_replacement(&self, _action: &Action<TGame>, _game: &TGame) -> Option<TGame::Action> {
        None
    }

    /// The given action has just been applied to the game state, this is this effect's chance to
    /// react to it.
    ///
    /// If this effect would like to perform another action in reaction to the observed one, it
    /// should add it to the game's staging action set.
    fn observe_action(
        &mut self,
        _action: &Action<TGame>,
        _game_state: &TGame,
        _emit_action: &mut dyn FnMut(ActionPayload<TGame>),
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
        _input: &PlayerInput<TGame>,
        _game_state: &TGame,
        _emit_action: &mut dyn FnMut(ActionPayload<TGame>),
    ) {
        panic!("Input being passed to an observer that has no consume_input implementation")
    }
}

pub trait Observer<TGame: GameDomain>: BaseObserver<TGame> {
    fn clone_box(&self) -> Box<dyn Observer<TGame>>;
}

impl<TGame: GameDomain, T: 'static + BaseObserver<TGame> + Clone> Observer<TGame> for T {
    fn clone_box(&self) -> Box<dyn Observer<TGame>> {
        Box::new(self.clone())
    }
}

impl<TGame: GameDomain> Clone for Box<dyn Observer<TGame>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
