use core::ids::ActionId;

/// The 5 colors of magic
///
/// Explicitly does not include "Colorless" or "Snow", as these are not colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

/// The set of possible constraints that can be placed on a single mana cost symbol
#[derive(Debug, Clone, Copy)]
pub enum ManaConstraint {
    Color(Color),
    Colorless,
    Snow,
}

/// A single component of a mana cost
///
/// Maps 1:1 to a single circular symbol in the mana cost on a printed mtg card
#[derive(Debug, Clone, Copy)]
pub enum BaseManaCostComponent {
    /// A fixed amount of generic mana
    ConcreteGeneric(u32),

    /// An 'X' amount of generic mana
    XGeneric,

    /// A single mana with the given constraint
    Single(ManaConstraint),

    /// A single phyrexian mana with the given constraint
    Phyrexian(ManaConstraint),
}

impl BaseManaCostComponent {
    fn converted_mana_cost(&self) -> u32 {
        use BaseManaCostComponent::*;
        match self {
            ConcreteGeneric(val) => *val,
            // NB: While an object with an XGeneric cost is on the stack, the CMC of this component
            // is actually the value of X chosen.
            XGeneric => 0,
            Single(_) | Phyrexian(_) => 1,
        }
    }
}

pub enum ManaCostComponent {
    /// A regular mana cost component
    Base(BaseManaCostComponent),

    /// A hybrid mana cost that could be either of two possibilities
    Hybrid(BaseManaCostComponent, BaseManaCostComponent),
}

impl ManaCostComponent {
    fn converted_mana_cost(&self) -> u32 {
        match self {
            ManaCostComponent::Base(a) => a.converted_mana_cost(),
            ManaCostComponent::Hybrid(a, b) => {
                // 202.3f When calculating the converted mana cost of an object with a hybrid mana
                //     symbol in its mana cost, use the largest component of each hybrid symbol.
                std::cmp::max(a.converted_mana_cost(), b.converted_mana_cost())
            }
        }
    }
}

pub struct ManaCost {
    pub components: Vec<ManaCostComponent>,
}

impl ManaCost {
    pub fn converted_mana_cost(&self) -> u32 {
        self.components
            .iter()
            .map(ManaCostComponent::converted_mana_cost)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct Mana {
    pub color: Option<Color>,

    /// The action that created this mana, to enable the imeplementation of certain rules/effects
    ///
    /// Some effects place additional constraints on how the mana can be used. Eg Ancient Ziggurat.
    /// These contraints apply even if the producing object no longer exists
    ///
    /// Effects that care about the object that produced the mana only (todo: verify) care about the
    /// state of that object when it produced that mana, not the state of the object when that mana
    /// is spent.
    /// Eg snow mana costs only care about whether the producer had the snow supertype when it
    /// produced it. It doesn't matter if the producing object loses the snow supertype before the
    /// mana is used.
    pub producer: Option<ActionId>,
}

#[derive(Debug, Clone)]
pub struct ManaPool {
    pub mana: Vec<Mana>,
}

impl ManaPool {
    pub fn total_of(&self, color: Option<Color>) -> u32 {
        self.mana.iter().filter(|m| m.color == color).count() as u32
    }
}
