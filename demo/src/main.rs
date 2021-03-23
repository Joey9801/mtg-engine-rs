use core::{
    actions::{Action, ActionPayload},
    game::Game,
    BaseObserver, PlayerInput, PlayerInputPayload,
};
use std::time::Instant;

use mtg::{
    game::{Mtg, MtgGameBuilder},
    player_inputs::{MtgInput, PriorityInput},
    steps::{CombatStep, Step, SubStep},
    PlayerId,
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

    // Helper so that passing priority below doesn't take nearly 10 lines of code each time
    fn pass_priority(game: &mut Game<Mtg>, player: PlayerId) {
        game.player_input(PlayerInput {
            source: player,
            payload: PlayerInputPayload::DomainInput(MtgInput::PriorityInput(
                PriorityInput::PassPriority,
            )),
        })
        .expect("Expected to succeed in giving input");
    }

    let sw = Instant::now();

    // Game starts in Alice's first main phase with Alice just about to receive priority
    game.tick_until_player_input();
    pass_priority(&mut game, alice);
    game.tick_until_player_input();
    pass_priority(&mut game, bob);
    game.tick_until_player_input();

    // After Alice and Bob both pass on an empty stack, the game should move to the StartOfCombat
    // step, and both Alice and Bob should get another round of priority
    assert_eq!(
        game.game_state.step.step,
        Step::Combat(CombatStep::StartOfCombat)
    );

    pass_priority(&mut game, alice);
    game.tick_until_player_input();
    pass_priority(&mut game, bob);
    game.tick_until_player_input();

    // After Alice and Bob both passing on an empty stack, the game should move to the
    // Alice's DeclareAttackers step. Alice has no creature to attack with, so must immediately
    // send the "Finished" input
    game.player_input(PlayerInput {
        source: alice,
        payload: PlayerInputPayload::DomainInput(MtgInput::Finished),
    })
    .expect("Expected to succeed in giving input");
    game.tick_until_player_input();

    // After declaring attackers, there is round of priority starting with the active player, Alice
    pass_priority(&mut game, alice);
    game.tick_until_player_input();
    pass_priority(&mut game, bob);
    game.tick_until_player_input();

    println!("Took {:?}", sw.elapsed());
}
