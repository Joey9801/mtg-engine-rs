pub mod steps;
pub mod zone;
pub mod base_rules;
pub mod game;
pub mod actions;

use actions::{Action, BaseAction};
use game::Game;
use zone::{ZoneId, ZoneLocation};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct PlayerId(usize);

impl std::fmt::Debug for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player:{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ObjectId(usize);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ObserverId(usize);

#[derive(Clone, Debug)]
pub struct IdGenerator<T> {
    counter: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> IdGenerator<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            _phantom: std::marker::PhantomData::<T>,
        }
    }
}

impl IdGenerator<PlayerId> {
    pub fn next_id(&mut self) -> PlayerId {
        let ret = PlayerId(self.counter);
        self.counter += 1;
        ret
    }
}

impl IdGenerator<ObserverId> {
    pub fn next_id(&mut self) -> ObserverId {
        let ret = ObserverId(self.counter);
        self.counter += 1;
        ret
    }
}

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
    // TODO
}

#[derive(Clone, Copy, Debug)]
pub enum Controller {
    Game,
    Player(PlayerId),
}

#[derive(Clone, Debug)]
pub enum ObjectReference {
    Concrete(ObjectId),
    Abstract(ZoneLocation),
}


pub trait BaseObserver: std::fmt::Debug {
    /// Who owns this effect.
    fn controller(&self) -> Controller { Controller::Game }
    
    /// Called once when the game allocates this observer its globally unique ID
    ///
    /// Any actions emitted by a given observer will have that observers ID attached to them by the
    /// game. This property can be used to safely implement internal state machines without
    /// accidentally reacting to actions emitted by different observers.
    fn set_id(&mut self, _id: ObserverId) {}
    
    /// If this observer is no longer relevant, returning false from this method will cause it to
    /// be cleaned up.
    fn alive(&self, _game: &Game) -> bool { true }
    
    /// The given action has just been applied to the game state, this is this effect's chance to
    /// react to it.
    ///
    /// If this effect would like to perform another action in reaction to the observed one, it
    /// should add it to the game's staging action set.
    fn observe_action(&mut self, action: &Action, game: &Game, emit_action: &mut dyn FnMut(BaseAction));
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
