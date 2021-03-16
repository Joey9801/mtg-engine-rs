use mtg_engine_core::ids::{ObjectId, PlayerId};

/// The 10 special actions defined in 116.2
#[derive(Clone, Copy, Debug)]
pub enum SpecialAction {
    /// 116.2a. Playing a land is a special action
    PlayLand,

    /// 116.2b. Turning a face-down creature face up is a special action.
    FlipCreature,

    /// 116.2c. Some effects allow a player to take an action at a later time, usually to end a
    /// continuous effect or to stop a delayed triggered ability from triggering.
    ///
    /// An example of this action is paying `{U}` to Quenching Fire
    /// ```text
    ///     Quenchable Fire {3}{R}
    ///     Sorcery
    ///     Quenchable Fire deals 3 damage to target player or planeswalker. It deals an additional
    ///     3 damage to that player or planeswalker at the beginning of your next upkeep step
    ///     unless that player or that planeswalker’s controller pays {U} before that step.
    /// ```
    EffectAction,

    /// 116.2d. Some effects from static abilities allow a player to take an action to ignore the
    /// effect from that ability for a duration. Doing so is a special action.
    IgnoreStatic,

    /// 116.2e. One card (Circling Vultures) has the ability “You may discard Circling Vultures any
    /// time you could cast an instant.” Doing so is a special action.
    CirclingVultures,

    /// 116.2f. A player who has a card with suspend in their hand may exile that card. This is a
    /// special action.
    SuspendCard,

    /// 116.2g. In a Planechase game, rolling the planar die is a special action.
    RollPlanarDie,

    /// 116.2h. In a Conspiracy Draft game, turning a face-down conspiracy card in the command zone
    /// face up is a special action.
    FlipConspiracy,
}

/// A thing that a player can choose to do while they hold priority
///
/// The contents of this enum do not necesarily contain all of the information required to execute
/// the given action. For the inputs that need further information, additional followup primitive
/// inputs are required.
#[derive(Clone, Copy, Debug)]
pub enum PriorityInput {
    /// Pass the priority to the next player
    ///
    /// Has no further inputs.
    PassPriority,

    /// Cast a spell
    ///
    /// Expects a single further input of ObjectId for the spell to cast.
    /// Additional inputs may be requested by the casting subsystem for properties such as:
    /// - Targets for the spell
    /// - Player chosen X variables on the card
    /// - Which mana to use to pay for the card
    CastSpell,

    /// Activate an ability of some game object
    ///
    /// Expects a single further input of AbilityId.
    /// Additional inputs may be requested based on the input chosen.
    ActivateAbility,

    /// Perform one of the 10 special actions described in section 116.2 of the comprehensive rules
    SpecialAction(SpecialAction),
}

/// The input type specific to the game of Magic
#[derive(Clone, Copy, Debug)]
pub enum MtgInput {
    /// When a player has priority, this variant of input is expected
    PriorityInput(PriorityInput),

    /// Any time the engine is expecting a game object, including but not limited to:
    /// - After choosing 'CastSpell' as a PriorityInput
    /// - When declaring a creature that should attack
    /// - When declaring a game object for a creature to attack
    /// - When declaring a game object as the target of a spell
    ObjectId(ObjectId),

    /// Any time the engine is expecting a player as an input, including but not limited to:
    /// - When declaring a player that a creature should attack
    /// - When declaring a player as the target of a spell
    PlayerId(PlayerId),

    /// Any time the engine is expecting an arbitrary integer as an input, including but not
    /// limited to:
    /// - When choosing some X value
    /// - When assigning combat damage to multiple objects
    Value(i32),

    /// General purpose input to declare that the player is finished giving inputs
    ///
    /// Can be used:
    /// - When all attackers have been set
    /// - When all blockers have been set
    /// - When a variable number of targets have finished being declared
    ///
    /// Is /not/ for passing priority, which is a separate specific input in
    /// [PriorityInput](enum.PriorityInput.html).
    Finished,
}
