pub mod base_rules;
pub mod action;
pub mod player_inputs;
pub mod steps;
pub mod zone;
pub mod game;

use action::MtgAction;
pub use core::ids::{ActionId, IdGenerator, ObserverId, PlayerId};
use core::{
    ids::{ObjectId, ZoneId},
};
use zone::ZoneLocation;


#[derive(Clone, Debug)]
pub struct SharedZones {
    pub battlefield: ZoneId,
    pub stack: ZoneId,
    pub exile: ZoneId,
    pub command: ZoneId,
    pub ante: ZoneId,
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
    pub owner: PlayerId,
    pub controller: PlayerId,

    /// The action to be executed if/when this object is resolved from the top of the stack.
    ///
    /// Only relevant for objects on the stack.
    /// This action will be added to the staging set and subject to replacement effects just like
    /// any other.
    pub resolve_action: Option<Box<dyn MtgAction>>,
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
