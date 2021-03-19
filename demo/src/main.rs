use core::{
    actions::{Action, ActionPayload},
    BaseObserver, PlayerInput, PlayerInputPayload,
};
use std::time::Instant;

use mtg::{
    game::{Mtg, MtgGameBuilder},
    player_inputs::{MtgInput, PriorityInput},
    steps::{Step, SubStep},
};

#[derive(Clone, Debug)]
struct StdoutDebugObserver {}

impl BaseObserver<Mtg> for StdoutDebugObserver {
    fn observe_action(
        &mut self,
        action: &Action<Mtg>,
        _game: &Mtg,
        _emit_action: &mut dyn FnMut(ActionPayload<Mtg>),
    ) {
        dbg!(action);
    }
}

fn main() {
    let mut game = MtgGameBuilder::new()
        .with_player("alice")
        .with_player("bob")
        .with_starting_life_total(20)
        .with_initial_step("alice", Step::PreCombatMain, SubStep::InProgress)
        .with_intial_priority("alice")
        .build();

    game.attach_observer(Box::new(StdoutDebugObserver {}));
    dbg!(&game);

    let alice = game.game_state.find_player("alice").unwrap();
    let bob = game.game_state.find_player("bob").unwrap();

    let sw = Instant::now();
    game.tick_until_player_input();

    assert_eq!(game.expecting_input_from(), Some(alice));
    game.player_input(PlayerInput {
        source: alice,
        payload: PlayerInputPayload::DomainInput(MtgInput::PriorityInput(
            PriorityInput::PassPriority,
        )),
    })
    .expect("Expected to succeed in giving input");
    assert_eq!(game.expecting_input_from(), None);

    game.tick_until_player_input();

    assert_eq!(game.expecting_input_from(), Some(bob));

    game.player_input(PlayerInput {
        source: bob,
        payload: PlayerInputPayload::DomainInput(MtgInput::PriorityInput(
            PriorityInput::PassPriority,
        )),
    })
    .expect("Expected to succeed in giving input");

    assert_eq!(game.expecting_input_from(), None);

    game.tick_until_player_input();
    
    println!("Took {:?}", sw.elapsed());
}
