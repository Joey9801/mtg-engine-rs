use std::{cmp::min, collections::HashMap};

use crate::Object;
use core::ids::{ObjectId, PlayerId, ZoneId};

#[derive(Clone, Copy, Debug)]
pub enum AbstractZoneLocation {
    Top,
    Bottom,
    NthFromTop(usize),
    NthFromBottom(usize),

    /// Only valid when referring to a destination in an unordered zone
    Undefined,
}

impl AbstractZoneLocation {
    fn implies_ordering(&self) -> bool {
        use AbstractZoneLocation::*;
        match self {
            Top | Bottom | NthFromTop(_) | NthFromBottom(_) => true,
            Undefined => false,
        }
    }
}

/// A way to describe a particular object by its zone location
#[derive(Clone, Copy, Debug)]
pub struct ZoneLocation {
    pub zone: ZoneId,
    pub loc: AbstractZoneLocation,
}

#[derive(Clone, Debug)]
pub struct Zone {
    /// Unique ID of this zone
    pub id: ZoneId,

    /// Human readable name for this zone
    pub name: String,

    /// The owner of this zone, or None if this is a shared zone
    pub owner: Option<PlayerId>,

    /// Are the contents of this zone public knowledge
    pub public: bool,

    storage: HashMap<ObjectId, Object>,

    /// If the order of the elements in this zone is relevant, that order.
    ///
    /// The first element of this vector is the "bottom" of the zone
    ordering: Option<Vec<ObjectId>>,
}

impl Zone {
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn resolve_abstract_zone_location(&self, loc: AbstractZoneLocation) -> Option<ObjectId> {
        assert!(loc.implies_ordering());
        assert!(self.ordering.is_some());

        let ordering = self.ordering.as_ref().unwrap();
        match loc {
            AbstractZoneLocation::Top => ordering.last(),
            AbstractZoneLocation::Bottom => ordering.first(),
            AbstractZoneLocation::NthFromTop(n) => ordering.iter().rev().nth(n),
            AbstractZoneLocation::NthFromBottom(n) => ordering.iter().nth(n),
            AbstractZoneLocation::Undefined => unreachable!(),
        }
        .cloned()
    }

    pub fn insert(&mut self, object: Object, loc: AbstractZoneLocation) {
        if let Some(ordering) = &mut self.ordering {
            assert!(loc.implies_ordering());
            let index = match loc {
                AbstractZoneLocation::Top => ordering.len(),
                AbstractZoneLocation::Bottom => 0,
                AbstractZoneLocation::NthFromTop(n) => ordering.len().saturating_sub(n),
                AbstractZoneLocation::NthFromBottom(n) => min(n, ordering.len()),
                AbstractZoneLocation::Undefined => unreachable!(),
            };
            ordering.insert(index, object.id);
        } else {
            assert!(!loc.implies_ordering());
        }

        self.storage.insert(object.id, object);
    }

    pub fn remove(&mut self, id: ObjectId) -> Option<Object> {
        assert!(self.storage.contains_key(&id));
        let obj = self.storage.remove(&id)?;

        if let Some(ordering) = self.ordering.as_mut() {
            let index = ordering
                .iter()
                .position(|&x| x == id)
                .expect("Object in ordered zone is missing from the ordering");
            ordering.remove(index);
        }

        Some(obj)
    }
    
    pub fn top(&self) -> Option<&Object> {
        if let Some(ordering) = &self.ordering {
            ordering
                .last()
                .map(|id| self.storage.get(id))
                .flatten()
        } else {
            None
        }
    }
}

pub enum NamedZone {
    Library(PlayerId),
    Hand(PlayerId),
    Graveyard(PlayerId),
    Battlefield,
    Stack,
    Exile,
    Command,
    Ante,
}

impl NamedZone {
    pub fn build(self, id: ZoneId) -> Zone {
        use NamedZone::*;

        let name = match self {
            Library(p) => format!("{}'s library", p),
            Hand(p) => format!("{}'s hand", p),
            Graveyard(p) => format!("{}'s graveyard", p),
            Battlefield => String::from("battlefield"),
            Stack => String::from("stack"),
            Exile => String::from("exile"),
            Command => String::from("command"),
            Ante => String::from("ante"),
        };

        let owner = match self {
            Library(p) | Hand(p) | Graveyard(p) => Some(p),
            _ => None,
        };

        let public = match self {
            Library(_) | Hand(_) => false,
            _ => true,
        };

        let storage = HashMap::new();

        let ordering = match self {
            Library(_) | Graveyard(_) | Stack => Some(Vec::new()),
            _ => None,
        };

        Zone {
            id,
            name,
            owner,
            public,
            storage,
            ordering,
        }
    }
}
