use std::hash::{Hash, Hasher};

use super::{province::Location, unit::{Unit, UnitType}};

pub trait Order {}

pub trait UnitOrder {}

impl Order for dyn UnitOrder {}



pub trait ComplexOrder {
    fn get_source(&self) -> &Unit;
}

// Player orders are orders that belong to a player rather than a unit e.g. builds.
// Builds are player orders because the unit does not yet exist.
// Disbands are player order because builds are.
pub enum PlayerOrder {
    Build(Location, UnitType),
    Disband(Location)
}

impl PartialEq for PlayerOrder {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Build(l0, l1), Self::Build(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Disband(l0), Self::Disband(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Hash for PlayerOrder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Build(l0, l1) => {
                l0.get_name().hash(state);
                l1.hash(state);
            }
            Self::Disband(l0) => l0.get_name().hash(state),
        };
    }
}

impl Eq for PlayerOrder {
    
}

//     def __str__(self):
//         return f"Build {self.unit_type.value} {self.location}"


//     def __str__(self):
//         return f"Disband {self.location}"


// class PlayerOrder(Order):
//     """Player orders are orders that belong to a player rather than a unit e.g. builds."""

//     def __init__(self, location: Location):
//         super().__init__()
//         self.location: Location = location

//     def __hash__(self):
//         return hash(self.location.name)

//     def __eq__(self, other):
//         return isinstance(other, type(self)) and self.location.name == other.location.name

// class Hold(UnitOrder):
//     def __init__(self):
//         super().__init__()

//     def __str__(self):
//         return "Holds"


// class Core(UnitOrder):
//     def __init__(self):
//         super().__init__()

//     def __str__(self):
//         return "Cores"


// class Move(UnitOrder):
//     def __init__(self, destination: Location):
//         super().__init__()
//         self.destination: Location = destination

//     def __str__(self):
//         return f"- {self.destination}"

// class ConvoyMove(UnitOrder):
//     def __init__(self, destination: Location):
//         super().__init__()
//         self.destination: Location = destination

//     def __str__(self):
//         return f"Convoys - {self.destination}"


// class ConvoyTransport(ComplexOrder):
//     def __init__(self, source: Unit, destination: Location):
//         super().__init__(source)
//         self.destination: Location = destination

//     def __str__(self):
//         return f"Convoys {self.source.province} - {self.destination}"


// class Support(ComplexOrder):
//     def __init__(self, source: Unit, destination: Location):
//         super().__init__(source)
//         self.destination: Location = destination

//     def __str__(self):
//         suffix = "Hold"

//         destination_province = self.destination
//         if isinstance(self.destination, Coast):
//             destination_province = self.destination.province

//         if self.source.province != destination_province:
//             suffix = f"- {self.destination}"
//         return f"Supports {self.source.province} {suffix}"


// class RetreatMove(UnitOrder):
//     def __init__(self, destination: Location):
//         super().__init__()
//         self.destination: Location = destination

//     def __str__(self):
//         return f"- {self.destination}"


// class RetreatDisband(UnitOrder):
//     def __init__(self):
//         super().__init__()

//     def __str__(self):
//         return f"Disbands"

