
use std::{cell::RefCell, fmt::Debug, sync::Arc};

use geo_types::Coord;
use geos::Geometry;

use super::{player::{Player, PlayerInfo}, unit::Unit};


impl PartialEq for Location {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

pub enum Location {
    Coast(CoastReference),
    Province(ProvinceReference)
}

impl Location {
    pub fn get_owner(&self) -> &Player {
        todo!()
    }

    pub fn get_name(&self) -> &str {
        todo!()
    }
}

// index version isn't currently used but kept for now
#[derive(Debug, Clone)]
pub enum CoastReference {
    Name(String),
    Index(usize)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProvinceReference {
    Name(String),
    Index(usize)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProvinceType {
    LAND,
    ISLAND,
    SEA
}

// todo implementing display is better
impl ProvinceType {
    pub fn name(&self) -> &str {
        match self {
            ProvinceType::LAND => "LAND",
            ProvinceType::ISLAND => "ISLAND",
            ProvinceType::SEA => "SEA",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Coords {
    pub all_locs: Vec<Coord<f64>>,
    pub all_rets: Vec<Coord<f64>>,
    pub primary_unit_coordinate: Option<Coord<f64>>,
    pub retreat_unit_coordinate: Option<Coord<f64>>,
}

pub struct ProvinceInfo {
    pub name: String,
    pub province_type: ProvinceType,
    pub adjacent: Vec<ProvinceReference>,
    pub has_supply_center: bool,
    pub initial_owner: Option<Arc<PlayerInfo>>,
    pub initial_core: Option<Arc<PlayerInfo>>,
    pub initial_unit: Option<Unit>,
    pub geometry: Option<Geometry>,
    pub coasts: Option<Vec<Coast>>,
    pub coords: Coords
}

impl ProvinceInfo {
    // This should only be called once all province adjacencies have been set.
    pub fn to_reference(&self) -> ProvinceReference {
        ProvinceReference::Name(self.name.clone())
    }

    pub fn resolve_reference(&self, coast: &CoastReference) -> &Coast {
        match coast {
            CoastReference::Name(name) => {
                for potential in self.coasts.as_ref().unwrap() {
                    if &potential.name == name {
                        return potential;
                    }
                }

                panic!("Unknown Coast Name for Reference {}", name)
            }
            CoastReference::Index(i) => {
                self.coasts.as_ref().and_then(|f| f.get(*i)).unwrap()
            }
        }
    }

    pub fn resolve_reference_mut(&mut self, coast: &CoastReference) -> &mut Coast {
        match coast {
            CoastReference::Name(name) => {
                for potential in self.coasts.as_mut().unwrap() {
                    if &potential.name == name {
                        return potential;
                    }
                }

                panic!("Unknown Coast Name for Reference {}", name)
            }
            CoastReference::Index(i) => {
                self.coasts.as_mut().and_then(|f| f.get_mut(*i)).unwrap()
            }
        }
    }

    pub fn set_coasts<'a, F>(&'a mut self, resolve: F) where 
        F: Fn(&ProvinceReference) -> &'a RefCell<ProvinceInfo>
     {
        // Externally set, i. e. by json_cheats()
        if self.coasts.is_some() {
            return
        }

        // seas don't have coasts
        if self.province_type == ProvinceType::SEA {
            self.coasts = Some(Vec::new());
            return;
        }

        let mut sea_provinces = Vec::new();
        for reference in &self.adjacent {
            let province = resolve(&reference).borrow();
            if province.province_type == ProvinceType::SEA || province.province_type == ProvinceType::ISLAND {
                sea_provinces.push(province.to_reference());
            }
        }

        if sea_provinces.len() > 0 {
            let name = format!("{} coast", self.name);
            self.coasts = Some(vec![
                Coast { 
                    name, 
                    adjacent_seas: sea_provinces, 
                    province: self.to_reference(),
                    coords: Coords { 
                        all_locs: Vec::new(), 
                        all_rets: Vec::new(),
                        primary_unit_coordinate: None, 
                        retreat_unit_coordinate: None
                    }
                }
            ])
        } else {
            self.coasts = Some(Vec::new())
        }
    }

}

impl Debug for ProvinceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvinceInfo")
            .field("name", &self.name)
            .field("province_type", &self.province_type)
            .field("adjacent", &self.adjacent)
            .field("has_supply_center", &self.has_supply_center)
            .field("initial_owner", &self.initial_owner)
            .field("local_unit", &self.initial_unit)
            .field("coasts", &self.coasts)
            .field("coords", &self.coords)
            .finish()
    }
}

#[derive(Debug)]
pub struct Province {
    pub info: Arc<ProvinceInfo>,
    pub corer: Option<Arc<PlayerInfo>>,
    pub core: Option<Arc<PlayerInfo>>,
    pub half_core: Option<Arc<PlayerInfo>>,
    pub owner: Option<Arc<PlayerInfo>>,
    pub unit: Option<Unit>,
    pub dislodged_unit: Option<Unit>
}
// class Province(Location):
//     def __init__(
//         self,
//         name: str,
//         province_type: ProvinceType,
//         has_supply_center: bool,
//         adjacent: set[Province],
//         coasts: set[Coast],
//         core: player.Player | None,
//         owner: player.Player | None,
//         local_unit: unit.Unit | None,  # TODO: probably doesn't make sense to init with a unit
//     ):
//         super().__init__(name, primary_unit_coordinate, retreat_unit_coordinate)
//         self.geometry: Polygon = coordinates
//         self.type: ProvinceType = province_type
//         self.has_supply_center: bool = has_supply_center
//         self.adjacent: set[Province] = adjacent
//         self.coasts: set[Coast] = coasts
//         self.corer: player.Player | None = None
//         self.core: player.Player | None = core
//         self.half_core: player.Player | None = None
//         self.owner: player.Player | None = owner
//         self.unit: unit.Unit | None = local_unit
//         self.dislodged_unit: unit.Unit | None = None

//     def __str__(self):
//         return self.name

//     def get_owner(self) -> player.Player | None:
//         return self.owner

//     def get_unit(self) -> unit.Unit | None:
//         return self.unit

//     def coast(self) -> Coast:
//         if len(self.coasts) != 1:
//             raise RuntimeError(f"Cannot get coast of a province with num coasts {len(self.coasts)} != 1")
//         return next(coast for coast in self.coasts)




// from __future__ import annotations

// from abc import abstractmethod
// from enum import Enum
// from typing import TYPE_CHECKING

// from shapely import Polygon, MultiPolygon

// if TYPE_CHECKING:
//     from diplomacy.persistence import player
//     from diplomacy.persistence import unit


// class Location:
//     def __init__(
//         self,
//         name: str,
//         primary_unit_coordinate: tuple[float, float],
//         retreat_unit_coordinate: tuple[float, float],
//     ):
//         self.all_locs = set()
//         self.all_rets = set()
//         self.name: str = name
//         self.primary_unit_coordinate: tuple[float, float] = primary_unit_coordinate
//         self.retreat_unit_coordinate: tuple[float, float] = retreat_unit_coordinate
//         if primary_unit_coordinate:
//             self.all_locs: set[tuple[float, float]] = {primary_unit_coordinate}
//         if retreat_unit_coordinate:
//             self.all_rets: set[float[float, float]] = {retreat_unit_coordinate}

//     @abstractmethod
//     def get_owner(self) -> player.Player | None:
//         pass

//     @abstractmethod
//     def get_unit(self) -> unit.Unit | None:
//         pass

//     def __str__(self):
//         return self.name



#[derive(Debug, Clone)]
pub struct Coast {
    pub name: String,
//         primary_unit_coordinate: tuple[float, float],
//         retreat_unit_coordinate: tuple[float, float],
    pub adjacent_seas: Vec<ProvinceReference>,
    pub province:  ProvinceReference,
    pub coords: Coords
}

impl Coast {
    // This should only be called once all province adjacencies have been set.
    pub fn to_reference(&self) -> CoastReference {
        CoastReference::Name(self.name.clone())
    }
}

// class Coast(Location):
//     def __init__(
//         self,
//         name: str,
//         primary_unit_coordinate: tuple[float, float],
//         retreat_unit_coordinate: tuple[float, float],
//         adjacent_seas: set[Province],
//         province: Province,
//     ):
//         super().__init__(name, primary_unit_coordinate, retreat_unit_coordinate)
//         self.adjacent_seas: set[Province] = adjacent_seas
//         self.province: Province = province

//     def __str__(self):
//         return self.name

//     def get_owner(self) -> player.Player | None:
//         return self.province.get_owner()

//     def get_unit(self) -> unit.Unit | None:
//         return self.province.get_unit()

//     def get_adjacent_coasts(self) -> set[Coast]:
//         # TODO: (BETA) this will generate false positives (e.g. mini province keeping 2 big province coasts apart)
//         adjacent_coasts: set[Coast] = set()
//         if self.province.type == ProvinceType.ISLAND:
//             for province2 in self.province.adjacent:
//                 adjacent_coasts.update(province2.coasts)
//             return adjacent_coasts
      
//         for province2 in self.province.adjacent:
//             for coast2 in province2.coasts:
//                 if self.adjacent_seas & coast2.adjacent_seas:
//                     adjacent_coasts.add(coast2)
//         return adjacent_coasts
