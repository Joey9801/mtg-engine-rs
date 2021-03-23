use std::collections::HashMap;

use crate::mana::{Color, ManaCost};

/// 205.2a
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CardType {
    Artifact,
    Conspiracy,
    Creature,
    Enchantment,
    Instant,
    Land,
    Phenomenon,
    Plane,
    Planeswalker,
    Scheme,
    Sorcery,
    Tribal,
    Vanguard,
}

/// 205.3g
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactType {
    Clue,
    Contraption,
    Equipment,
    Food,
    Fortification,
    Gold,
    Treasure,
    Vehicle,
}

/// 205.3h
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnchantmentType {
    Aura,
    Cartouche,
    Curse,
    Rune,
    Saga,
    Shard,
    Shrine,
}

/// 205.3i
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LandType {
    Desert,
    Forest,
    Gate,
    Island,
    Lair,
    Locus,
    Mine,
    Mountain,
    Plains,
    PowerPlant,
    Swamp,
    Tower,
    Urzas,
}

impl LandType {
    pub fn is_basic(&self) -> bool {
        use LandType::*;
        match self {
            Forest | Island | Mountain | Plains | Swamp => true,
            _ => false,
        }
    }
}

/// 205.3j
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlaneswalkerType {
    Ajani,
    Aminatou,
    Angrath,
    Arlinn,
    Ashiok,
    Basri,
    Bolas,
    Calix,
    Chandra,
    Dack,
    Daretti,
    Davriel,
    Domri,
    Dovin,
    Elspeth,
    Estrid,
    Freyalise,
    Garruk,
    Gideon,
    Huatli,
    Jace,
    Jaya,
    Jeska,
    Karn,
    Kasmina,
    Kaya,
    Kiora,
    Koth,
    Liliana,
    Lukka,
    Nahiri,
    Narset,
    Niko,
    Nissa,
    Nixilis,
    Oko,
    Ral,
    Rowan,
    Saheeli,
    Samut,
    Sarkhan,
    Serra,
    Sorin,
    Szat,
    Tamiyo,
    Teferi,
    Teyo,
    Tezzeret,
    Tibalt,
    Tyvar,
    Ugin,
    Venser,
    Vivien,
    Vraska,
    Will,
    Windgrace,
    Wrenn,
    Xenagos,
    Yanggu,
    Yanling,
}

/// 205.3k
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellType {
    Adventure,
    Arcane,
    Trap,
}

/// 205.3m
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreatureType {
    Advisor,
    Aetherborn,
    Ally,
    Angel,
    Antelope,
    Ape,
    Archer,
    Archon,
    Army,
    Artificer,
    Assassin,
    AssemblyWorker,
    Atog,
    Aurochs,
    Avatar,
    Azra,
    Badger,
    Barbarian,
    Basilisk,
    Bat,
    Bear,
    Beast,
    Beeble,
    Berserker,
    Bird,
    Blinkmoth,
    Boar,
    Bringer,
    Brushwagg,
    Camarid,
    Camel,
    Caribou,
    Carrier,
    Cat,
    Centaur,
    Cephalid,
    Chimera,
    Citizen,
    Cleric,
    Cockatrice,
    Construct,
    Coward,
    Crab,
    Crocodile,
    Cyclops,
    Dauthi,
    Demigod,
    Demon,
    Deserter,
    Devil,
    Dinosaur,
    Djinn,
    Dog,
    Dragon,
    Drake,
    Dreadnought,
    Drone,
    Druid,
    Dryad,
    Dwarf,
    Efreet,
    Egg,
    Elder,
    Eldrazi,
    Elemental,
    Elephant,
    Elf,
    Elk,
    Eye,
    Faerie,
    Ferret,
    Fish,
    Flagbearer,
    Fox,
    Frog,
    Fungus,
    Gargoyle,
    Germ,
    Giant,
    Gnome,
    Goat,
    Goblin,
    God,
    Golem,
    Gorgon,
    Graveborn,
    Gremlin,
    Griffin,
    Hag,
    Harpy,
    Hellion,
    Hippo,
    Hippogriff,
    Homarid,
    Homunculus,
    Horror,
    Horse,
    Human,
    Hydra,
    Hyena,
    Illusion,
    Imp,
    Incarnation,
    Insect,
    Jackal,
    Jellyfish,
    Juggernaut,
    Kavu,
    Kirin,
    Kithkin,
    Knight,
    Kobold,
    Kor,
    Kraken,
    Lamia,
    Lammasu,
    Leech,
    Leviathan,
    Lhurgoyf,
    Licid,
    Lizard,
    Manticore,
    Masticore,
    Mercenary,
    Merfolk,
    Metathran,
    Minion,
    Minotaur,
    Mole,
    Monger,
    Mongoose,
    Monk,
    Monkey,
    Moonfolk,
    Mouse,
    Mutant,
    Myr,
    Mystic,
    Naga,
    Nautilus,
    Nephilim,
    Nightmare,
    Nightstalker,
    Ninja,
    Noble,
    Noggle,
    Nomad,
    Nymph,
    Octopus,
    Ogre,
    Ooze,
    Orb,
    Orc,
    Orgg,
    Otter,
    Ouphe,
    Ox,
    Oyster,
    Pangolin,
    Peasant,
    Pegasus,
    Pentavite,
    Pest,
    Phelddagrif,
    Phoenix,
    Phyrexian,
    Pilot,
    Pincher,
    Pirate,
    Plant,
    Praetor,
    Prism,
    Processor,
    Rabbit,
    Rat,
    Rebel,
    Reflection,
    Rhino,
    Rigger,
    Rogue,
    Sable,
    Salamander,
    Samurai,
    Sand,
    Saproling,
    Satyr,
    Scarecrow,
    Scion,
    Scorpion,
    Scout,
    Sculpture,
    Serf,
    Serpent,
    Servo,
    Shade,
    Shaman,
    Shapeshifter,
    Shark,
    Sheep,
    Siren,
    Skeleton,
    Slith,
    Sliver,
    Slug,
    Snake,
    Soldier,
    Soltari,
    Spawn,
    Specter,
    Spellshaper,
    Sphinx,
    Spider,
    Spike,
    Spirit,
    Splinter,
    Sponge,
    Squid,
    Squirrel,
    Starfish,
    Surrakar,
    Survivor,
    Tentacle,
    Tetravite,
    Thalakos,
    Thopter,
    Thrull,
    Treefolk,
    Trilobite,
    Triskelavite,
    Troll,
    Turtle,
    Unicorn,
    Vampire,
    Vedalken,
    Viashino,
    Volver,
    Wall,
    Warlock,
    Warrior,
    Weird,
    Werewolf,
    Whale,
    Wizard,
    Wolf,
    Wolverine,
    Wombat,
    Worm,
    Wraith,
    Wurm,
    Yeti,
    Zombie,
    Zubera,
}

/// 205.3n
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlanarType {
    Alara,
    Arkhos,
    Azgol,
    Belenon,
    BolassMeditationRealm,
    Dominaria,
    Equilor,
    Ergamon,
    Fabacin,
    Innistrad,
    Iquatana,
    Ir,
    Kaldheim,
    Kamigawa,
    Karsus,
    Kephalai,
    Kinshala,
    Kolbahan,
    Kyneth,
    Lorwyn,
    Luvion,
    Mercadia,
    Mirrodin,
    Moag,
    Mongseng,
    Muraganda,
    NewPhyrexia,
    Phyrexia,
    Pyrulea,
    Rabiah,
    Rath,
    Ravnica,
    Regatha,
    Segovia,
    SerrasRealm,
    Shadowmoor,
    Shandalar,
    Ulgrotha,
    Valla,
    Vryn,
    Wildfire,
    Xerex,
    Zendikar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubType {
    Artifact(ArtifactType),
    Enchantment(EnchantmentType),
    Land(LandType),
    Planeswalker(PlaneswalkerType),
    Spell(SpellType),
    Creature(CreatureType),
    Planar(PlanarType),
}

impl From<ArtifactType> for SubType {
    fn from(v: ArtifactType) -> Self {
        Self::Artifact(v)
    }
}

impl From<EnchantmentType> for SubType {
    fn from(v: EnchantmentType) -> Self {
        Self::Enchantment(v)
    }
}

impl From<LandType> for SubType {
    fn from(v: LandType) -> Self {
        Self::Land(v)
    }
}

impl From<PlaneswalkerType> for SubType {
    fn from(v: PlaneswalkerType) -> Self {
        Self::Planeswalker(v)
    }
}

impl From<SpellType> for SubType {
    fn from(v: SpellType) -> Self {
        Self::Spell(v)
    }
}

impl From<CreatureType> for SubType {
    fn from(v: CreatureType) -> Self {
        Self::Creature(v)
    }
}

impl From<PlanarType> for SubType {
    fn from(v: PlanarType) -> Self {
        Self::Planar(v)
    }
}

impl SubType {
    /// Is this subtype correlated with the given card type
    pub fn correlated(&self, card_type: CardType) -> bool {
        match (card_type, self) {
            (CardType::Artifact, SubType::Artifact(_)) => true,
            (CardType::Creature, SubType::Creature(_)) => true,
            (CardType::Enchantment, SubType::Enchantment(_)) => true,
            (CardType::Land, SubType::Land(_)) => true,
            (CardType::Plane, SubType::Planar(_)) => true,
            (CardType::Planeswalker, SubType::Planeswalker(_)) => true,
            (CardType::Sorcery, SubType::Spell(_)) => true,
            (CardType::Instant, SubType::Spell(_)) => true,
            _ => false,
        }
    }
}

impl CardType {
    pub fn correlated(&self, sub_type: SubType) -> bool {
        sub_type.correlated(*self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuperType {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CardTypeLine {
    pub super_types: Vec<SuperType>,
    pub card_types: Vec<CardType>,
    pub sub_types: Vec<SubType>,
}

pub trait HasType<T> {
    fn has_type(&self, t: T) -> bool;
}

impl HasType<SuperType> for CardTypeLine {
    fn has_type(&self, t: SuperType) -> bool {
        self.super_types.iter().any(|st| *st == t)
    }
}

impl HasType<CardType> for CardTypeLine {
    fn has_type(&self, t: CardType) -> bool {
        self.card_types.iter().any(|ct| *ct == t)
    }
}

impl HasType<SubType> for CardTypeLine {
    fn has_type(&self, t: SubType) -> bool {
        self.sub_types.iter().any(|st| *st == t)
    }
}

macro_rules! define_has_subtype {
    ($name:ident) => {
        impl HasType<$name> for CardTypeLine {
            fn has_type(&self, t: $name) -> bool {
                self.has_type(SubType::from(t))
            }
        }
    };
}

define_has_subtype!(ArtifactType);
define_has_subtype!(EnchantmentType);
define_has_subtype!(LandType);
define_has_subtype!(PlaneswalkerType);
define_has_subtype!(SpellType);
define_has_subtype!(CreatureType);
define_has_subtype!(PlanarType);

#[derive(Clone, Debug, Default)]
pub struct CardRules {
    /// The oracle text for the card as it appears on gatherer
    pub text: String,
    // TODO: A structured view of the rules text that the engine can actually use
}

/// A literal definition of a card as it would appear in real life
///
/// This structure only contains information that is able to
#[derive(Clone, Debug, Default)]
pub struct CardDefinition {
    pub name: String,
    pub mana_cost: ManaCost,
    pub color_indicator: Vec<Color>,
    pub type_line: CardTypeLine,
    pub text: String,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub hand_modifier: Option<i32>,
    pub life_modifier: Option<i32>,
}

impl<T> HasType<T> for CardDefinition
where
    CardTypeLine: HasType<T>,
{
    fn has_type(&self, t: T) -> bool {
        self.type_line.has_type(t)
    }
}

pub struct CardUniverse {
    /// Maps card name to its definition
    pub cards: HashMap<String, CardDefinition>,
}

impl CardUniverse {
    pub fn new_empty() -> Self {
        Self {
            cards: HashMap::new(),
        }
    }

    pub fn add_card(&mut self, defn: CardDefinition) {
        self.cards.insert(defn.name.clone(), defn);
    }

    pub fn find_by_name<S: AsRef<str>>(&self, name: S) -> Option<&CardDefinition> {
        self.cards.get(name.as_ref())
    }
}

// Temporary method for testing, until a proper way of storing card definition data is settled upon
pub fn make_card_universe() -> CardUniverse {
    let mut universe = CardUniverse::new_empty();

    // Add the 5 basic land types
    universe.add_card(CardDefinition {
        name: "Forest".to_string(),
        type_line: CardTypeLine {
            super_types: vec![SuperType::Basic],
            card_types: vec![CardType::Land],
            sub_types: vec![SubType::Land(LandType::Forest)],
        },
        ..Default::default()
    });
    universe.add_card(CardDefinition {
        name: "Island".to_string(),
        type_line: CardTypeLine {
            super_types: vec![SuperType::Basic],
            card_types: vec![CardType::Land],
            sub_types: vec![SubType::Land(LandType::Island)],
        },
        ..Default::default()
    });
    universe.add_card(CardDefinition {
        name: "Mountain".to_string(),
        type_line: CardTypeLine {
            super_types: vec![SuperType::Basic],
            card_types: vec![CardType::Land],
            sub_types: vec![SubType::Land(LandType::Mountain)],
        },
        ..Default::default()
    });
    universe.add_card(CardDefinition {
        name: "Plains".to_string(),
        type_line: CardTypeLine {
            super_types: vec![SuperType::Basic],
            card_types: vec![CardType::Land],
            sub_types: vec![SubType::Land(LandType::Plains)],
        },
        ..Default::default()
    });
    universe.add_card(CardDefinition {
        name: "Swamp".to_string(),
        type_line: CardTypeLine {
            super_types: vec![SuperType::Basic],
            card_types: vec![CardType::Land],
            sub_types: vec![SubType::Land(LandType::Swamp)],
        },
        ..Default::default()
    });

    universe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_type() {
        let universe = make_card_universe();

        let forest = universe
            .find_by_name("Forest")
            .expect("Expect Forest to be in the card universe");

        assert!(forest.type_line.has_type(SuperType::Basic));
        assert!(forest.type_line.has_type(CardType::Land));
        assert!(forest.type_line.has_type(LandType::Forest));
        assert!(!forest.type_line.has_type(CardType::Creature));
        assert!(!forest.type_line.has_type(CreatureType::Wizard));

        assert!(forest.has_type(SuperType::Basic));
        assert!(forest.has_type(CardType::Land));
        assert!(forest.has_type(LandType::Forest));
        assert!(!forest.has_type(CardType::Creature));
        assert!(!forest.has_type(CreatureType::Wizard));
    }
}
