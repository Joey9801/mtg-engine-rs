use std::{any::Any, hash::BuildHasherDefault};

use crate::{
    game::{self, GameState},
    ids::PlayerId,
    steps::{GameStep, Step, SubStep},
    zone::ZoneLocation,
    ObjectReference,
};

pub trait BaseMtgAction: std::fmt::Debug + std::any::Any {
    fn apply(&self, game_state: &mut GameState);
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: BaseMtgAction> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait MtgAction: BaseMtgAction + AsAny {
    fn clone_box(&self) -> Box<dyn MtgAction>;
}

impl<T: 'static + BaseMtgAction + Clone> MtgAction for T {
    fn clone_box(&self) -> Box<dyn MtgAction> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn MtgAction> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait MtgActionDowncast {
    fn downcast_ref<T: BaseMtgAction>(&self) -> Option<&T>;

    fn is<T: BaseMtgAction>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }
}

impl MtgActionDowncast for Box<dyn MtgAction> {
    fn downcast_ref<T: BaseMtgAction>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

#[derive(Clone, Debug)]
pub struct CompositeAction {
    pub tag: &'static str,
    pub components: Vec<Box<dyn MtgAction>>,
}

impl BaseMtgAction for CompositeAction {
    fn apply(&self, game_state: &mut GameState) {
        for sub_action in &self.components {
            sub_action.apply(game_state);
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdvanceStep {
    pub new_step: Step,
    pub new_substep: SubStep,
    pub new_active_player: PlayerId,
}

impl BaseMtgAction for AdvanceStep {
    fn apply(&self, game_state: &mut GameState) {
        game_state.step = GameStep {
            active_player: self.new_active_player,
            step: self.new_step,
            substep: self.new_substep,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetPriority {
    pub new_priority: PlayerId,
}

impl BaseMtgAction for SetPriority {
    fn apply(&self, game_state: &mut GameState) {
        game_state.priority = Some(self.new_priority);
    }
}

#[derive(Clone, Debug)]
pub struct PassPriority;

impl BaseMtgAction for PassPriority {
    fn apply(&self, game_state: &mut GameState) {
        game_state.priority = None;
    }
}

#[derive(Clone, Debug)]
pub struct ChangeObjectZone {
    pub obj_ref: ObjectReference,
    pub new_loc: ZoneLocation,
}

impl BaseMtgAction for ChangeObjectZone {
    fn apply(&self, game_state: &mut GameState) {
        let obj = match self.obj_ref {
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
                .get_mut(&self.new_loc.zone)
                .expect("Failed to find zone in game state")
                .insert(obj, self.new_loc.loc)
        }
    }
}
