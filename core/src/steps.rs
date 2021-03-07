use crate::PlayerId;

/// StartingSteps aren't technically steps in the game, but are defined here so that the start of a
/// game can leverage the same state transition machinery as the main body of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartingStep {
    /// Pseudo-step that the game starts up in
    ///
    /// This step exists such that there is a transition into the first real state that observers
    /// can react to.
    Init,
    
    /// It is during this step that the turn order is initially set
    ///
    /// During this step the "active player" is meaningless
    ChoosingTurnOrder,
    
    /// This step includes all mulligan choices
    ///
    /// The active player during this step is the player making mulligan choices
    InitialHandDraw
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeginningStep {
    Untap,
    Upkeep,
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatStep {
    StartOfCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndOfCombat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndStep {
    EndOfTurn,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Starting(StartingStep),
    Beginning(BeginningStep),
    PreCombatMain,
    Combat(CombatStep),
    PostCombatMain,
    End(EndStep),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubStep {
    InProgress,
    Ending,
}

#[derive(Clone, Debug)]
pub struct GameStep {
    pub active_player: PlayerId,
    pub step: Step,
    pub substep: SubStep,
}