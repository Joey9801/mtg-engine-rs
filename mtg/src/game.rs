use std::collections::HashMap;

use core::{
    game::{ActionQueue, GameDomain, GameTimestamp},
    ids::{IdGenerator, ObserverId, PlayerId, ZoneId},
};

use crate::{
    action::MtgAction,
    player_inputs::MtgInput,
    steps::{GameStep, StartingStep, Step, SubStep},
    zone::{NamedZone, Zone},
    Player, SharedZones,
};

#[derive(Clone, Debug)]
pub struct Mtg {
    /// Set of players in turn order
    pub players: HashMap<PlayerId, Player>,

    /// Turn order stored as a map of CurrentPLayer -> NextPlayer
    pub turn_order: HashMap<PlayerId, PlayerId>,

    pub step: GameStep,
    pub priority: Option<PlayerId>,
    pub zones: HashMap<ZoneId, Zone>,
    pub shared_zones: SharedZones,
}

impl GameDomain for Mtg {
    type Input = MtgInput;
    type Action = Box<dyn MtgAction>;
}

impl Mtg {
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

    pub fn find_player<S: AsRef<str>>(&self, name: S) -> Option<PlayerId> {
        self.players
            .values()
            .filter(|p| p.name == name.as_ref())
            .map(|p| p.id)
            .next()
    }
}

pub struct MtgGameBuilder {
    players: HashMap<PlayerId, Player>,
    step: Option<GameStep>,
    priority: Option<PlayerId>,
    zones: HashMap<ZoneId, Zone>,
    shared_zones: SharedZones,
    starting_life_total: i32,

    player_id_gen: IdGenerator<PlayerId>,
    zone_id_gen: IdGenerator<ZoneId>,
}

impl MtgGameBuilder {
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

    pub fn with_initial_step<S: AsRef<str>>(
        mut self,
        player_name: S,
        step: Step,
        substep: SubStep,
    ) -> Self {
        let pid = self
            .players
            .iter()
            .find(|(_pid, player)| &player.name == player_name.as_ref())
            .map(|(pid, _player)| pid)
            .cloned()
            .expect("Couldn't find player with name");

        self.step = Some(GameStep {
            active_player: pid,
            step,
            substep,
        });

        self
    }

    pub fn with_intial_priority<S: AsRef<str>>(mut self, name: S) -> Self {
        let pid = self
            .players
            .iter()
            .find(|(_pid, player)| &player.name == name.as_ref())
            .map(|(pid, _player)| pid)
            .cloned()
            .expect("Couldn't find player with name");
        self.priority = Some(pid);
        self
    }

    pub fn build(self) -> core::game::Game<Mtg> {
        assert!(self.players.len() > 0);

        let step = match self.step {
            Some(s) => s,
            None => GameStep {
                active_player: *self.players.keys().next().unwrap(),
                step: Step::Starting(StartingStep::Init),
                substep: SubStep::InProgress,
            },
        };

        let mut players = self.players.values().collect::<Vec<_>>();
        players.sort_by_key(|p| p.name.as_str());

        let mut turn_order = HashMap::new();
        for i in 0..(players.len() - 1) {
            turn_order.insert(players[i].id, players[i + 1].id);
        }
        turn_order.insert(players[players.len() - 1].id, players[0].id);

        let mut observer_id_gen = IdGenerator::<ObserverId>::new();
        let self_id = observer_id_gen.next_id();

        let mut game = core::game::Game {
            game_state: Mtg {
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
        };

        crate::base_rules::attach(&mut game);

        game
    }
}
