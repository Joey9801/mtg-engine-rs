use std::collections::HashMap;

use crate::{
    ids::{IdGenerator, ObjectId, ObserverId, PlayerId, ZoneId},
    steps::{GameStep, StartingStep, Step, SubStep},
    zone::{NamedZone, Zone},
    Action, Observer, Player,
};

#[derive(Clone, Debug)]
pub struct SharedZones {
    pub battlefield: ZoneId,
    pub stack: ZoneId,
    pub exile: ZoneId,
    pub command: ZoneId,
    pub ante: ZoneId,
}

#[derive(Clone, Debug)]
pub struct Game {
    /// Set of players in turn order
    pub players: HashMap<PlayerId, Player>,

    /// The first turn of the game is taken by the first element of this field.
    pub turn_order: Vec<PlayerId>,

    pub step: GameStep,
    pub priority: Option<PlayerId>,
    pub zones: HashMap<ZoneId, Zone>,
    pub shared_zones: SharedZones,

    /// Actions that have been recieved, but have not been passed through the replacement effect machinery
    pub staging_actions: Vec<Action>,

    /// Actions which are ready to execute
    pub pending_actions: Vec<Action>,

    pub observer_id_gen: IdGenerator<ObserverId>,
    pub observers: HashMap<ObserverId, Box<dyn Observer>>,
}

#[derive(Clone, Debug)]
pub enum TickResult {
    Ticked(Action),
    NeedPlayerInput(PlayerId),
}

impl Game {
    pub fn stack(&self) -> &Zone {
        self.zones
            .get(&self.shared_zones.stack)
            .expect("Can't find the stack")
    }

    pub fn stack_mut(&mut self) -> &mut Zone {
        self.zones
            .get_mut(&self.shared_zones.stack)
            .expect("Can't find the stack")
    }

    pub fn battlefield(&self) -> &Zone {
        self.zones
            .get(&self.shared_zones.battlefield)
            .expect("Can't find the battlefield")
    }

    pub fn battlefield_mut(&mut self) -> &mut Zone {
        self.zones
            .get_mut(&self.shared_zones.battlefield)
            .expect("Can't find the battlefield")
    }

    pub fn exile(&self) -> &Zone {
        self.zones
            .get(&self.shared_zones.exile)
            .expect("Can't find the exile zone")
    }

    pub fn exile_mut(&mut self) -> &mut Zone {
        self.zones
            .get_mut(&self.shared_zones.exile)
            .expect("Can't find the exile zone")
    }

    pub fn command(&self) -> &Zone {
        self.zones
            .get(&self.shared_zones.command)
            .expect("Can't find the command zone")
    }

    pub fn command_mut(&mut self) -> &mut Zone {
        self.zones
            .get_mut(&self.shared_zones.command)
            .expect("Can't find the command zone")
    }

    pub fn ante(&self) -> &Zone {
        self.zones
            .get(&self.shared_zones.ante)
            .expect("Can't find the ante zone")
    }

    pub fn ante_mut(&mut self) -> &mut Zone {
        self.zones
            .get_mut(&self.shared_zones.ante)
            .expect("Can't find the ante zone")
    }

    fn requires_player_input(&self) -> bool {
        // TODO: Determine whether the staging actions need player input to order correctly
        // TODO: Return a more descriptive value that defines which player input is required from, and what sort of input is required
        self.pending_actions.len() + self.staging_actions.len() == 0
    }

    fn promote_staged_actions(&mut self) {
        debug_assert!(self.pending_actions.is_empty());
        debug_assert!(!self.staging_actions.is_empty());

        // TODO: Resolve replacement effects for each staged action
        // TODO: Establish the correct ordering for actions (may require player input)

        while let Some(action) = self.staging_actions.pop() {
            self.pending_actions.push(action);
        }
    }

    /// Attempt to perform a single action
    fn tick(&mut self) -> TickResult {
        if self.pending_actions.is_empty() {
            if !self.staging_actions.is_empty() {
                self.promote_staged_actions();
            } else {
                return TickResult::NeedPlayerInput(self.step.active_player);
            }
        }

        let action = self
            .pending_actions
            .pop()
            .expect("Unexpectedly empty pending action set");

        action.base_action.apply(self);

        // Perform this slightly strange dance to avoid problems associated with the immutable
        // reference to the game state containing the a reference to the mutable receiver of
        // `o.observe_action`.
        // Since the observers are hidden behind trait objects, no observer should be able to
        // inspect the internal state of another anyway, so presenting them a game state with no
        // observers should have no effect on any functionality.
        // New actions are placed into a temporary vector for the same reason.
        let mut observers = std::mem::take(&mut self.observers);
        let mut new_actions = Vec::new();
        for (oid, o) in observers.iter_mut() {
            let controller = o.controller();
            o.observe_action(&action, self, &mut |action| {
                new_actions.push((action, controller, *oid))
            });
        }
        self.observers = observers;
        for (base_action, controller, source) in new_actions {
            self.staging_actions.push(Action {
                base_action,
                controller,
                source,
                original: None,
            });
        }

        TickResult::Ticked(action)
    }

    pub fn tick_until_player_input(&mut self) {
        while !self.requires_player_input() {
            self.tick();
        }
    }

    pub fn find_player<S: AsRef<str>>(&self, name: S) -> Option<PlayerId> {
        self.players
            .values()
            .filter(|p| p.name == name.as_ref())
            .map(|p| p.id)
            .next()
    }

    pub fn attach_observer(&mut self, mut o: Box<dyn Observer>) {
        let id = self.observer_id_gen.next_id();
        o.set_id(id);
        self.observers.insert(id, o);
    }
}

pub struct GameBuilder {
    players: HashMap<PlayerId, Player>,
    step: Option<GameStep>,
    priority: Option<PlayerId>,
    zones: HashMap<ZoneId, Zone>,
    shared_zones: SharedZones,
    staging_actions: Vec<Action>,
    pending_actions: Vec<Action>,
    starting_life_total: i32,

    player_id_gen: IdGenerator<PlayerId>,
    zone_id_gen: IdGenerator<ZoneId>,
    implicit_turn_order: bool,
}

impl GameBuilder {
    pub fn new() -> Self {
        let player_id_gen = IdGenerator::<PlayerId>::new();
        let mut zone_id_gen = IdGenerator::<ZoneId>::new();

        let mut zones = HashMap::new();

        // Insert the default shared zones
        let battlefield_id = zone_id_gen.next_id();
        zones.insert(battlefield_id, NamedZone::Battlefield.build(battlefield_id));

        let stack_id = zone_id_gen.next_id();
        zones.insert(stack_id, NamedZone::Stack.build(stack_id));

        let exile_id = zone_id_gen.next_id();
        zones.insert(exile_id, NamedZone::Exile.build(exile_id));

        let command_id = zone_id_gen.next_id();
        zones.insert(command_id, NamedZone::Command.build(command_id));

        let ante_id = zone_id_gen.next_id();
        zones.insert(ante_id, NamedZone::Ante.build(ante_id));

        let shared_zones = SharedZones {
            battlefield: battlefield_id,
            stack: stack_id,
            exile: exile_id,
            command: command_id,
            ante: ante_id,
        };

        Self {
            players: HashMap::new(),
            step: None,
            priority: None,
            zones,
            shared_zones,
            staging_actions: Vec::new(),
            pending_actions: Vec::new(),
            starting_life_total: 20,
            player_id_gen,
            zone_id_gen,
            implicit_turn_order: false,
        }
    }

    pub fn with_starting_life_total(mut self, x: i32) -> Self {
        for p in self.players.values_mut() {
            p.life_total = x;
        }
        self.starting_life_total = x;

        self
    }

    pub fn with_player<S: AsRef<str>>(mut self, name: S) -> Self {
        let player_id = self.player_id_gen.next_id();

        let library_id = self.zone_id_gen.next_id();
        let hand_id = self.zone_id_gen.next_id();
        let graveyard_id = self.zone_id_gen.next_id();

        self.zones
            .insert(library_id, NamedZone::Library(player_id).build(library_id));
        self.zones
            .insert(hand_id, NamedZone::Hand(player_id).build(hand_id));
        self.zones.insert(
            graveyard_id,
            NamedZone::Graveyard(player_id).build(graveyard_id),
        );

        let player = Player {
            id: player_id,
            name: name.as_ref().to_string(),
            life_total: self.starting_life_total,
            library: library_id,
            hand: hand_id,
            graveyard: graveyard_id,
        };
        self.players.insert(player_id, player);

        self
    }

    pub fn with_initial_step(mut self, step: GameStep) -> Self {
        self.step = Some(step);
        self
    }

    /// Creates an implicit turn order by sorting the players on their name
    pub fn with_implicit_turn_order(mut self) -> Self {
        self.implicit_turn_order = true;
        self
    }

    pub fn build(self) -> Game {
        assert!(self.players.len() > 0);

        let step = match self.step {
            Some(s) => s,
            None => GameStep {
                active_player: *self.players.keys().next().unwrap(),
                step: Step::Starting(StartingStep::Init),
                substep: SubStep::InProgress,
            },
        };

        let turn_order = if self.implicit_turn_order {
            let mut players = self.players.values().collect::<Vec<_>>();
            players.sort_by_key(|p| p.name.as_str());
            players.iter().map(|p| p.id).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Game {
            players: self.players,
            turn_order,
            step,
            priority: self.priority,
            zones: self.zones,
            shared_zones: self.shared_zones,
            staging_actions: self.staging_actions,
            pending_actions: self.pending_actions,
            observer_id_gen: IdGenerator::new(),
            observers: HashMap::new(),
        }
    }
}
