// import logging

// from diplomacy.persistence.phase import Phase
// from diplomacy.persistence.player import Player
// from diplomacy.persistence.province import Province, Coast, Location
// from diplomacy.persistence.unit import Unit, UnitType

// logger = logging.getLogger(__name__)

use std::{collections::HashMap, sync::Arc};

use serenity::all::GuildId;

use super::{phase::Phase, player::{Player, PlayerInfo}, province::{Province, ProvinceInfo}};

pub struct BoardInfo {
    pub name: String,
    pub players: Vec<Arc<PlayerInfo>>,
    pub name_to_info: HashMap<String, Arc<ProvinceInfo>>,
    pub datafile: String
}


pub struct Board {
    info: BoardInfo,
    players: Vec<Player>,
    info_to_province: HashMap<String, Province>,
    // units: HashSet<RefCell<Unit>>,
    phase: Phase,
    year: i64,
    // TODO this is sad
    pub board_id: Option<GuildId>,
    pub fish: i64,
    orders_enabled: bool,
    // data: todo!(),
}

impl Board {
    pub fn new(info: BoardInfo, phase: Phase) -> Board {
        let mut info_to_province = HashMap::new();

        for info in info.name_to_info.values() {
            let province = Province {
                info: Arc::clone(info),
                corer: None,
                core: info.initial_core.clone(),
                half_core: None,
                owner: info.initial_owner.clone(),
                unit: info.initial_unit.clone(),
                dislodged_unit: None,
            };
            info_to_province.insert(info.name.clone(), province);
        }

        let players = info.players.iter().map(|info| {
            Player::new(Arc::clone(info), Vec::new())
        }).collect();

        // TODO init player center count
        
        Board {
            info,
            players,
            info_to_province,
            phase,
            year: 0,
            board_id: None,
            fish: 0,
            orders_enabled: true,
        }
    }

    // TODO: we could have this as a dict ready on the variant
    fn get_player(&self, name: &str) -> Option<&Player> {
        // we ignore capitalization because this is primarily used for user input

        for player in self.players.iter() {
            if player.info.name.to_lowercase() == name.to_lowercase() {
                return Some(player)
            }
        }

        None
    }

    fn get_players_by_score(&self) -> Vec<Player> {
        todo!()
    }


    // TODO: we could have this as a dict ready on the variant
    fn get_province(&self, name: &str) -> Option<&Province> {
        // we ignore capitalization because this is primarily used for user input

        for province in self.info_to_province.values() {
            if province.info.name.to_lowercase() == name.to_lowercase() {
                return Some(province)
            }
        }

        None
    }

//     def get_province_and_coast(self, name: str) -> tuple[Province, Coast | None]:
//         # TODO: (BETA) we build this everywhere, let's just have one live on the Board on init
//         # we ignore capitalization because this is primarily used for user input
//         name = name.lower()
//         name_to_province: dict[str, Province] = {}
//         name_to_coast: dict[str, Coast] = {}
//         for province in self.provinces:
//             name_to_province[province.name.lower()] = province
//             for coast in province.coasts:
//                 name_to_coast[coast.name.lower()] = coast

//         coast = name_to_coast.get(name)
//         if coast:
//             return coast.province, coast
//         else:
//             return name_to_province[name], None

//     def get_location(self, name: str) -> Location:
//         province, coast = self.get_province_and_coast(name)
//         if coast:
//             return coast
//         return province

//     def get_build_counts(self) -> list[tuple[str, int]]:
//         build_counts = []
//         for player in self.players:
//             build_counts.append((player.name, len(player.centers) - len(player.units)))
//         build_counts = sorted(build_counts, key=lambda counts: counts[1])
//         return build_counts

//     def get_phase_and_year_string(self):
//         return f"{self.year} {self.phase.name}"

//     def change_owner(self, province: Province, player: Player):
//         if province.has_supply_center:
//             if province.owner:
//                 province.owner.centers.remove(province)
//             if player:
//                 player.centers.add(province)
//         province.owner = player

//     def create_unit(
//         self,
//         unit_type: UnitType,
//         player: Player,
//         province: Province,
//         coast: Coast | None,
//         retreat_options: set[Province] | None,
//     ) -> Unit:
//         unit = Unit(unit_type, player, province, coast, retreat_options)
//         if retreat_options is not None:
//             province.dislodged_unit = unit
//         else:
//             province.unit = unit
//         player.units.add(unit)
//         self.units.add(unit)
//         return unit

//     def move_unit(self, unit: Unit, new_location: Location) -> Unit:
//         new_province = new_location
//         new_coast = None
//         if isinstance(new_location, Coast):
//             new_province = new_location.province
//             new_coast = new_location

//         if new_province.unit:
//             raise RuntimeError(f"{new_province.name} already has a unit")
//         new_province.unit = unit
//         unit.province.unit = None
//         unit.province = new_province
//         unit.coast = new_coast
//         return unit

//     def delete_unit(self, province: Province) -> Unit:
//         unit = province.unit
//         province.unit = None
//         unit.player.units.remove(unit)
//         self.units.remove(unit)
//         return unit

//     def delete_dislodged_unit(self, province: Province) -> Unit:
//         unit = province.dislodged_unit
//         province.dislodged_unit = None
//         unit.player.units.remove(unit)
//         self.units.remove(unit)
//         return unit

//     def delete_all_units(self) -> None:
//         for unit in self.units:
//             unit.province.unit = None

//         for player in self.players:
//             player.units = set()

//         self.units = set()

//     def delete_dislodged_units(self) -> None:
//         dislodged_units = set()
//         for unit in self.units:
//             if unit.retreat_options:
//                 dislodged_units.add(unit)

//         for unit in dislodged_units:
//             unit.province.dislodged_unit = None
//             unit.player.units.remove(unit)
//             self.units.remove(unit)
}