use core::{
    actions::{Action, ActionPayload, EngineAction},
    game::Game,
    PlayerInput, PlayerInputPayload,
};
use cursive::{
    event::Key,
    traits::{Boxable, Finder, Nameable, Scrollable},
    views::{
        Dialog, EditView, LinearLayout, Panel, SelectView, TextView, ViewRef,
    },
    Cursive,
};
use mtg::{
    action::{AdvanceStep, MtgAction, MtgActionDowncast, PassPriority, SetPriority},
    game::{Mtg, MtgGameBuilder},
    player_inputs::{MtgInput, PriorityInput},
    steps::{Step, SubStep},
};
use std::ops::DerefMut;

fn build_new_game() -> Game<Mtg> {
    MtgGameBuilder::new()
        .with_player("alice")
        .with_player("bob")
        .with_starting_life_total(20)
        .with_initial_step("alice", Step::PreCombatMain, SubStep::InProgress)
        .with_intial_priority("alice")
        .build()
}

struct UiData {
    game: Option<Game<Mtg>>,
    action_history: Vec<Action<Mtg>>,
}

impl UiData {
    fn new() -> Self {
        Self {
            game: None,
            action_history: Vec::new(),
        }
    }

    fn new_game(&mut self) {
        self.game = Some(build_new_game());
        self.action_history = Vec::new();
    }
}

fn render_domain_action(action: &Box<dyn MtgAction>) -> String {
    if let Some(a) = action.as_t::<PassPriority>() {
        format!("{} passing priority", a.player)
    } else if let Some(a) = action.as_t::<SetPriority>() {
        format!("Setting priority to {}", a.new_priority)
    } else if let Some(a) = action.as_t::<AdvanceStep>() {
        format!(
            "Advance step to {}/{:?}/{:?}",
            a.new_active_player, a.new_step, a.new_substep
        )
    } else {
        format!("Missing custom renderer: {:?}", action)
    }
}

fn render_action(action: &Action<Mtg>) -> String {
    match &action.payload {
        ActionPayload::EngineAction(ea) => match ea {
            EngineAction::NoActions => String::from("-- No action signal --"),
            EngineAction::EndInput => String::from("-- End input --"),
            EngineAction::RequestInput(req) => format!(
                "-- Request input ({} -> {}) --",
                req.from_player, action.source
            ),
            EngineAction::PickReplacement(_) => {
                String::from("-- ambiguous replacement resolution --")
            }
            EngineAction::PickNextAction(_) => String::from("-- ambiguous ordering resolution --"),
        },
        ActionPayload::DomainAction(da) => render_domain_action(&da),
        ActionPayload::Composite(_) => String::from(" -- Composite action --"),
    }
}

fn create_game_view(siv: &mut Cursive) {
    let mut _view: ViewRef<LinearLayout> = siv
        .find_name("game-view")
        .expect("Cannot find main game view");
    let view: &mut LinearLayout = _view.deref_mut();

    // Nuke anything already inside the view
    while view.len() > 0 {
        view.remove_child(view.len() - 1);
    }

    view.add_child(Panel::new(
        LinearLayout::horizontal()
            .child(
                LinearLayout::vertical()
                    .child(
                        Panel::new(TextView::new("").with_name("current-step").min_height(5))
                            .title("Current game step"),
                    )
                    .child(
                        Panel::new(TextView::new("").with_name("action-queue").min_height(5))
                            .title("Action queue"),
                    )
                    .child(
                        Panel::new(TextView::new("").with_name("observers").min_height(5))
                            .title("Observers"),
                    )
                    .full_width(),
            )
            .child(
                Panel::new(
                    LinearLayout::vertical()
                        .child(
                            SelectView::<Action<Mtg>>::new()
                                .on_select(|siv, action| {
                                    siv.call_on_name(
                                        "action-history-focus",
                                        |focus_view: &mut TextView| {
                                            focus_view.set_content_wrap(true);
                                            focus_view.set_content(format!("{:#?}", action))
                                        },
                                    );
                                })
                                .with_name("action-history")
                                .full_height()
                                .scrollable(),
                        )
                        .child(Panel::new(
                            TextView::new("").with_name("action-history-focus"),
                        )),
                )
                .title("Action history"), // .fixed_width(60),
            )
            .full_height(),
    ));
}

fn update_game_view(siv: &mut Cursive) {
    let data = siv
        .take_user_data::<UiData>()
        .expect("Couldn't find game state");

    let game = match &data.game {
        Some(game) => game,
        None => {
            siv.set_user_data(data);
            return;
        }
    };

    let mut _view: ViewRef<LinearLayout> = siv
        .find_name("game-view")
        .expect("Cannot find main game view");
    let view: &mut LinearLayout = _view.deref_mut();

    let last_action = view
        .call_on_name("action-history", |v: &mut SelectView<Action<Mtg>>| {
            for new_action in &data.action_history[v.len()..] {
                v.add_item(render_action(new_action), new_action.clone());
            }
            if v.len() > 0 {
                v.set_selection(v.len() - 1);
                v.get_item(v.len() - 1)
                    .map(|(_label, action)| action.clone())
            } else {
                None
            }
        })
        .flatten();

    if let Some(action) = last_action {
        siv.call_on_name("action-history-focus", |focus_view: &mut TextView| {
            focus_view.set_content_wrap(true);
            focus_view.set_content(format!("{:#?}", action))
        });
    }

    view.call_on_name("current-step", |v: &mut TextView| {
        v.set_content(format!(
            "Step = {:#?}\nPriority = {:?}",
            game.game_state.step, game.game_state.priority
        ))
    });

    view.call_on_name("action-queue", |v: &mut TextView| {
        v.set_content(format!("{:#?}", game.action_queue))
    });

    view.call_on_name("observers", |v: &mut TextView| {
        v.set_content(format!("{:#?}", game.observers))
    });

    // Remember to put the game state back where it came from
    siv.set_user_data(data);
}

fn tick_once(siv: &mut Cursive) {
    let res = siv
        .with_user_data(|data: &mut UiData| {
            if let Some(game) = &mut data.game {
                match game.tick() {
                    core::game::TickResult::Ticked(action) => {
                        data.action_history.push(action);
                        Ok(())
                    }
                    core::game::TickResult::NeedPlayerInput => Err("Can't tick, need player input"),
                    core::game::TickResult::Stalled => Err("Game has stalled"),
                }
            } else {
                Err("Can't tick, no game in progress")
            }
        })
        .expect("Missing UI data");

    if let Err(msg) = res {
        siv.add_layer(
            Dialog::around(TextView::new(msg))
                .title("Error")
                .dismiss_button("Ok"),
        );
    }
    update_game_view(siv);
}

fn new_game(siv: &mut Cursive) {
    siv.with_user_data(|data: &mut UiData| data.new_game());
    create_game_view(siv);
    update_game_view(siv);
}

fn process_input(siv: &mut Cursive, input_str: &str) {
    let err = |siv: &mut Cursive, msg: &str| {
        siv.add_layer(
            Dialog::around(TextView::new(msg))
                .title("Error")
                .dismiss_button("Ok"),
        );
    };

    siv.pop_layer();

    let mut data = siv
        .take_user_data::<UiData>()
        .expect("Couldn't find game state");

    let game = match &mut data.game {
        Some(game) => game,
        None => {
            siv.set_user_data(data);
            return;
        }
    };

    let input_payload = match input_str {
        "pass" => {
            PlayerInputPayload::DomainInput(MtgInput::PriorityInput(PriorityInput::PassPriority))
        }
        _ => {
            err(siv, &format!("Unrecognized input: \"{}\"", input_str));
            siv.set_user_data(data);
            return;
        }
    };

    let input_player = game.expecting_input_from().unwrap();

    let res = game.player_input(PlayerInput {
        source: input_player,
        payload: input_payload,
    });

    if let Err(e) = res {
        err(siv, &format!("Bad input: {:#?}", e));
    }

    siv.set_user_data(data);
}

fn game_input_dialog(siv: &mut Cursive) {
    let err = |siv: &mut Cursive, msg: &str| {
        siv.add_layer(
            Dialog::around(TextView::new(msg))
                .title("Error")
                .dismiss_button("Ok"),
        );
    };

    let mut data = siv
        .take_user_data::<UiData>()
        .expect("Couldn't find game state");

    let game = match &mut data.game {
        Some(game) => game,
        None => {
            err(siv, "No game in progress");
            siv.set_user_data(data);
            return;
        }
    };

    let input_request = match &game.current_input_session {
        Some(s) => s.request.clone(),
        None => {
            err(siv, "No input currently expected");
            siv.set_user_data(data);
            return;
        }
    };

    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!(
                    "Input type: {}",
                    input_request.input_type
                )))
                .child(TextView::new(format!(
                    "For player: {}",
                    input_request.from_player
                )))
                .child(
                    EditView::new()
                        .on_submit(process_input)
                        .with_name("input-field"),
                ),
        )
        .title("Input"),
    );

    siv.set_user_data(data)
}

fn main() {
    let mut siv = cursive::default();

    // The menubar is a list of (label, menu tree) pairs.
    siv.menubar()
        // We add a new "File" tree
        .add_leaf("New game (F4)", new_game)
        .add_leaf("Tick once (F5)", tick_once)
        .add_leaf("Provide input (F6)", game_input_dialog)
        .add_delimiter()
        .add_leaf("Quit (q)", |s| s.quit());

    siv.set_autohide_menu(false);
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.add_global_callback(Key::F4, new_game);
    siv.add_global_callback(Key::F5, tick_once);
    siv.add_global_callback(Key::F6, game_input_dialog);
    siv.add_global_callback('q', |s| s.quit());

    siv.add_fullscreen_layer(LinearLayout::vertical().with_name("game-view"));

    siv.set_user_data(UiData::new());

    siv.run();
}
