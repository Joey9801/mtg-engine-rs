use mtg_engine_core::{
    actions::{Action, ActionPayload, MtgAction},
    base_rules,
    game::{GameBuilder, GameState},
    BaseObserver, Controller,
};

#[derive(Clone, Debug)]
struct StdoutDebugObserver {}

impl BaseObserver for StdoutDebugObserver {
    fn observe_action(
        &mut self,
        action: &Action,
        _game: &GameState,
        _emit_action: &mut dyn FnMut(ActionPayload),
    ) {
        dbg!(action);
    }
}

fn main() {
    let mut game = GameBuilder::new()
        .with_player("alice")
        .with_player("bob")
        .with_starting_life_total(20)
        .with_implicit_turn_order()
        .build();

    base_rules::attach(&mut game);
    game.attach_observer(Box::new(StdoutDebugObserver {}));

    dbg!(&game);

    let alice = game.find_player("alice").unwrap();
    let bob = game.find_player("bob").unwrap();
    let fake_oid = game.observer_id_gen.next_id();

    game.action_queue.add(Action {
        payload: ActionPayload::DomainAction(MtgAction::SetPriority(alice)),
        controller: Controller::Game,
        source: fake_oid,
        id: game.action_id_gen.next_id(),
        original: None,
        generated_at: game.game_timestamp,
    });
    game.tick_until_player_input();
    game.action_queue.add(Action {
        payload: ActionPayload::DomainAction(MtgAction::PassPriority),
        controller: Controller::Player(alice),
        source: fake_oid,
        id: game.action_id_gen.next_id(),
        original: None,
        generated_at: game.game_timestamp,
    });
    game.tick_until_player_input();
    game.action_queue.add(Action {
        payload: ActionPayload::DomainAction(MtgAction::SetPriority(bob)),
        controller: Controller::Game,
        source: fake_oid,
        id: game.action_id_gen.next_id(),
        original: None,
        generated_at: game.game_timestamp,
    });
    game.tick_until_player_input();
}
