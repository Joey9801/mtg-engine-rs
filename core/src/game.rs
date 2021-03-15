use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{
    actions::{ActionPayload, EngineAction, InputRequest},
    ids::{ActionId, IdGenerator, ObserverId, PlayerId, ZoneId},
    steps::{GameStep, StartingStep, Step, SubStep},
    zone::{NamedZone, Zone},
    Action, Controller, Observer, Player, PlayerInput,
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
pub struct GameState {
    /// Set of players in turn order
    pub players: HashMap<PlayerId, Player>,

    /// The first turn of the game is taken by the first element of this field.
    pub turn_order: Vec<PlayerId>,

    pub step: GameStep,
    pub priority: Option<PlayerId>,
    pub zones: HashMap<ZoneId, Zone>,
    pub shared_zones: SharedZones,
}

#[derive(Clone, Debug)]
pub struct ActionReplacementState {
    /// The action which is currently being considered for replacement
    pub subject: Action,

    /// The candidate actions which could repalce the subject
    pub candidates: Vec<Action>,

    /// The set of observers which have already had a candidate replacement accepted during this replacement chain
    pub used_observers: Vec<ObserverId>,
}

/// Sets of actions in various stages of processing
///
/// In principal, all actions flow through each of the fields in turn. In practice some of the
/// fields may be skipped internally in cases where there is no ambiguity.
#[derive(Clone, Debug, Default)]
pub struct ActionQueue {
    /// Actions that have been recieved, but have had no further processing
    pub received: Vec<Action>,

    /// The state of the current partially complete/ambiguous replacement chain
    pub partially_resolved_state: Option<ActionReplacementState>,

    /// Actions for which all replacements have been fully resolved, but have not yet been put in order for execution
    pub resolved: Vec<Action>,

    /// The current set of actions for which ordering must be determined
    pub staging: Vec<Action>,

    /// Queue of actions to actually execute, fully resolved and in order
    pub pending: VecDeque<Action>,
}

#[derive(Clone, Debug)]
enum ActionQueueStatus {
    /// There is an action in the partially resolved area that has multiple equally viable
    /// candidate replacements
    AmbiguousReplacements,

    /// There are N > 1 actions in the staging area for which ordering could not be decided
    AmbiguousOrdering,

    /// There are no actions anywhere in the queue
    Empty,

    /// The queue has actions ready to pop off and execute
    Ready,
}

impl ActionQueue {
    fn new() -> Self {
        Default::default()
    }

    /// There are no actions anywhere in the queue
    fn is_empty(&self) -> bool {
        self.received.is_empty()
            && self.partially_resolved_state.is_none()
            && self.resolved.is_empty()
            && self.staging.is_empty()
            && self.pending.is_empty()
    }

    /// Make a best-effort attempt to process the actions in this queue such that they become ready
    /// to execute.
    ///
    /// Requires an id generator as candidate action replacements must each be assigned their own ID.
    fn process(
        &mut self,
        id_gen: &mut IdGenerator<ActionId>,
        observers: &HashMap<ObserverId, Box<dyn Observer>>,
        game_state: &GameState,
    ) -> ActionQueueStatus {
        if self.partially_resolved_state.is_some() {
            return ActionQueueStatus::AmbiguousReplacements;
        }

        if !self.staging.is_empty() {
            return ActionQueueStatus::AmbiguousOrdering;
        }

        while let Some(original) = self.received.pop() {
            let mut original_rc: Option<Rc<Action>> = None;

            let mut candidate_replacements = Vec::new();
            for (oid, observer) in observers {
                if let Some(candidate) = observer.propose_replacement(&original, game_state) {
                    original_rc = match original_rc {
                        Some(o) => Some(o),
                        None => Some(Rc::new(original.clone())),
                    };

                    candidate_replacements.push(Action {
                        payload: ActionPayload::DomainAction(candidate),
                        controller: observer.controller(),
                        source: *oid,
                        id: id_gen.next_id(),
                        generated_at: original.generated_at,
                        original: original_rc.clone(),
                    });
                }
            }

            if candidate_replacements.len() == 0 {
                self.resolved.push(original);
            } else if candidate_replacements.len() == 1 {
                self.resolved.push(candidate_replacements.pop().unwrap());
            } else {
                self.partially_resolved_state = Some(ActionReplacementState {
                    subject: original,
                    candidates: candidate_replacements,
                    used_observers: Vec::new(),
                });
                return ActionQueueStatus::AmbiguousReplacements;
            }
        }

        // TODO: Any sort of attempt to sort the resolved action set, rather than just smashing
        // every resolved action into the pending set in whatever order it happens to be in.
        while let Some(action) = self.resolved.pop() {
            self.pending.push_back(action);
        }

        // By this point all actions should be fully resolved, ordered, and ready to execute
        debug_assert!(self.received.is_empty());
        debug_assert!(self.partially_resolved_state.is_none());
        debug_assert!(self.resolved.is_empty());
        debug_assert!(self.staging.is_empty());

        if self.pending.is_empty() {
            ActionQueueStatus::Empty
        } else {
            ActionQueueStatus::Ready
        }
    }

    pub fn add(&mut self, action: Action) {
        self.received.push(action);
    }

    /// Attempt to retrieve the next ready-to-execute action from the queue
    pub fn pop_next(&mut self) -> Option<Action> {
        if self.partially_resolved_state.is_some()
            || !self.resolved.is_empty()
            || !self.staging.is_empty()
        {
            None
        } else {
            self.pending.pop_front()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GameTimestamp(usize);

impl GameTimestamp {
    fn zero() -> Self {
        Self(0)
    }

    fn increment(&mut self) {
        self.0 += 1
    }
}

#[derive(Clone, Debug)]
pub struct InputSession {
    /// The original input request
    request: InputRequest,

    /// The observer that requested this input session, to which each input will be sent
    handler: ObserverId,
}

pub enum InputError {
    /// The observer managing the current input session rejected the input with the given message
    Rejected(String),

    /// The input was given when no input was being requested
    NoInputSession,

    /// The input came from the wrong player
    ///
    /// In this error case the input is not passed to the observer managing the current input session
    WrongPlayer,

    /// Internal error for when an observer has requested an input session, but has not defined an
    /// input handler.
    UnimplementedObserver,
}

#[derive(Clone, Debug)]
pub struct Game {
    /// Actual state of the game being run
    ///
    /// Fields of Game other than this are considered implementation details of the engine, and not
    /// properties of the game being simulated.
    pub game_state: GameState,

    /// Incremented every time something happens to the game
    ///
    /// Things which do happen in some sequential order because of real-world processing
    /// limitations but logically do happen at the same time will be marked as happeneing at the
    /// same game timestamp. For example if multiple observers emit an action in response to some
    /// other action being executed, each of those emitted actions will have the same
    /// 'generated_at' timestamp.
    pub game_timestamp: GameTimestamp,
    pub action_queue: ActionQueue,
    pub observer_id_gen: IdGenerator<ObserverId>,
    pub action_id_gen: IdGenerator<ActionId>,
    pub self_id: ObserverId,

    /// Storage for all obververs currently alive
    ///
    /// TODO: Not all observers have have implementations for each method in the trait.  This might
    /// be made more efficient by storing which subsets of observers need, eg, `observer_action`
    /// calling on them, and which observers doing so would be a waste of time.
    /// Actually benchmark this in real-world cases though, the cost of maintaining the sets +
    /// dictionary lookups for every key might be more than the cost of just calling the dummy
    /// default implementations of the methods.
    pub observers: HashMap<ObserverId, Box<dyn Observer>>,

    pub current_input_session: Option<InputSession>,
}

#[derive(Clone, Debug)]
pub enum TickResult {
    /// The game ticked normally
    Ticked(Action),

    /// The game requires player input before it can progress any further
    NeedPlayerInput,

    /// Error condition: The game has run out of actions to perform, and no observer has requested
    /// a player input
    Stalled,
}

impl GameState {
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
}

impl Game {
    fn apply_action(&mut self, action: &Action) {
        match &action.payload {
            ActionPayload::EngineAction(EngineAction::NoActions) => (),
            ActionPayload::EngineAction(EngineAction::RequestInput(request)) => {
                debug_assert!(self.current_input_session.is_none());
                self.current_input_session = Some(InputSession {
                    handler: action.source,
                    request: request.clone(),
                });
            }
            ActionPayload::EngineAction(EngineAction::EndInput) => (),
            ActionPayload::EngineAction(EngineAction::PickNextAction(_)) => todo!(),
            ActionPayload::EngineAction(EngineAction::PickReplacement(_)) => todo!(),
            ActionPayload::DomainAction(da) => da.apply(&mut self.game_state),
        }
    }

    /// Broadcast the given action to all observers and add any actions emitted in reaction to the
    /// staging set
    fn broadcast_action(&mut self, action: &Action) {
        // Explicit references to fields of self, so that the overzealous closure borrow rules
        // don't freak out about it containing references to `self`.
        let action_queue = &mut self.action_queue;
        let action_id_gen = &mut self.action_id_gen;
        let timestamp = self.game_timestamp;

        for (oid, o) in self.observers.iter_mut() {
            let controller = o.controller();
            o.observe_action(action, &self.game_state, &mut |reacting_action| {
                action_queue.add(Action {
                    payload: reacting_action,
                    controller,
                    source: *oid,
                    id: action_id_gen.next_id(),
                    original: None,
                    generated_at: timestamp,
                });
            });
        }
    }

    /// Attempt to perform a single action
    fn tick(&mut self) -> TickResult {
        if self.current_input_session.is_some() {
            return TickResult::NeedPlayerInput;
        }

        match self
            .action_queue
            .process(&mut self.action_id_gen, &self.observers, &self.game_state)
        {
            ActionQueueStatus::AmbiguousReplacements => {
                todo!("Player input to choose between competing replacement effects")
            }
            ActionQueueStatus::AmbiguousOrdering => todo!("Player input to order actions"),
            ActionQueueStatus::Ready => {
                let action = self
                    .action_queue
                    .pop_next()
                    .expect("Unexpectedly empty pending action set");
                self.apply_action(&action);
                self.broadcast_action(&action);
                self.game_timestamp.increment();
                TickResult::Ticked(action)
            }
            ActionQueueStatus::Empty => {
                // Generate a dummy game action to let the observers know that the game ticked while empty
                let action = Action {
                    payload: ActionPayload::EngineAction(EngineAction::NoActions),
                    controller: Controller::Game,
                    source: self.self_id,
                    id: self.action_id_gen.next_id(),
                    original: None,
                    generated_at: self.game_timestamp,
                };
                self.broadcast_action(&action);
                self.game_timestamp.increment();

                // If the queue is still empty after broadcasting the first NoActions, we're in the
                // stalled error state
                if self.action_queue.is_empty() {
                    TickResult::Stalled
                } else {
                    TickResult::Ticked(action)
                }
            }
        }
    }

    pub fn player_input(&mut self, input: PlayerInput) -> Result<(), InputError> {
        let curr_session = match &self.current_input_session {
            None => Err(InputError::NoInputSession)?,
            Some(session) => session,
        };

        if curr_session.request.from_player != input.source {
            Err(InputError::WrongPlayer)?
        }
        let handler_id = curr_session.handler;

        let handler = self
            .observers
            .get_mut(&curr_session.handler)
            .expect("Input session handler does not exist");

        let mut emitted_actions = Vec::new();
        handler.consume_input(&input, &self.game_state, &mut |action| {
            emitted_actions.push(action)
        });

        // Immediately apply and broadcast each of the emitted actions
        for action_payload in emitted_actions {
            let action_id = self.action_id_gen.next_id();
            let action = Action {
                payload: action_payload,
                controller: Controller::Player(input.source),
                source: handler_id,
                id: action_id,
                generated_at: self.game_timestamp,
                original: None,
            };
            self.apply_action(&action);
            self.broadcast_action(&action);
        }

        Ok(())
    }

    pub fn tick_until_player_input(&mut self) {
        while let TickResult::Ticked(_) = self.tick() {}
    }

    pub fn find_player<S: AsRef<str>>(&self, name: S) -> Option<PlayerId> {
        self.game_state
            .players
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

        let mut observer_id_gen = IdGenerator::<ObserverId>::new();
        let self_id = observer_id_gen.next_id();

        Game {
            game_state: GameState {
                players: self.players,
                turn_order,
                step,
                priority: self.priority,
                zones: self.zones,
                shared_zones: self.shared_zones,
            },
            action_id_gen: IdGenerator::new(),
            action_queue: ActionQueue::new(),
            observer_id_gen,
            observers: HashMap::new(),
            self_id,
            game_timestamp: GameTimestamp::zero(),
            current_input_session: None,
        }
    }
}
