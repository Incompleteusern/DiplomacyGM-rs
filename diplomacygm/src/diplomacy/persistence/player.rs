use std::{collections::HashSet, sync::Arc};

use super::{order::PlayerOrder, province::ProvinceInfo,};

#[derive(Debug)]
pub struct PlayerInfo {
    pub name: String,
    pub color: String,
    pub vscc: i64,
    pub iscc: i64,
}

pub struct Player {
    pub info: Arc<PlayerInfo>,
    centers: HashSet<Arc<ProvinceInfo>>,
    build_orders: HashSet<PlayerOrder>
}

impl PlayerInfo {
    pub fn new(name: String, color: String, vscc: i64, iscc: i64) -> Arc<PlayerInfo> {
        Arc::new(PlayerInfo { name, color, vscc, iscc })
    }
}

impl Player {
    pub fn new(info: Arc<PlayerInfo>, iscc: i64, centers: HashSet<Arc<ProvinceInfo>>) -> Player {
        Player {
            info, centers, build_orders: HashSet::new()
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