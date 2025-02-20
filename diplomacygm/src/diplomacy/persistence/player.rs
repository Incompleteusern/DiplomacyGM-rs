use std::{collections::HashSet, sync::Weak};

use super::{order::PlayerOrder, province::Province, unit::Unit};

pub struct Player {
    pub name: String,
    pub color: String,
    pub vscc: usize,
    pub iscc: usize,
    centers: HashSet<Weak<Province>>,
    units: HashSet<Weak<Unit>>,
    build_orders: HashSet<PlayerOrder>
}

impl Player {
    pub fn new(name: String, color: String, vscc: usize, iscc: usize, centers: HashSet<Weak<Province>>, units: HashSet<Weak<Unit>>) -> Player {
        Player {
            name, color, vscc, iscc, centers, units, build_orders: HashSet::new()
        }
    }
}

//     def __str__(self):
//         return self.name

//     def score(self):
//         if len(self.centers) > self.iscc:
//             return (len(self.centers) - self.iscc) / (self.vscc - self.iscc)
//         else:
//             return (len(self.centers) / self.iscc) - 1