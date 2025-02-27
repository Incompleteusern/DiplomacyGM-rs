// import copy
// import itertools
// import json
// import logging
// import time
// import numpy as np
// from typing import Callable
// from xml.etree.ElementTree import Element

// import shapely
// from lxml import etree
// from shapely.geometry import Point

// from diplomacy.map_parser.vector.transform import get_transform
// from diplomacy.map_parser.vector.utils i`mport get_player, get_unit_coordinates, get_svg_element, parse_path
// from diplomacy.persistence import phase
// from diplomacy.persistence.board import Board
// from diplomacy.persistence.player import Player
// from diplomacy.persistence.province import Province, ProvinceType, Coast
// from diplomacy.persistence.unit import Unit, UnitType

// # TODO: (BETA) all attribute getting should be in utils which we import and call utils.my_unit()
// # TODO: (BETA) consistent in bracket formatting


// logger = logging.getLogger(__name__)

use std::{borrow::Cow, collections::HashMap, env, fs::{self, File}, io::{BufRead, BufReader, BufWriter, Write}, path::PathBuf, sync::{Arc, Mutex}};

use geos::{Geom, Geometry};
use quick_xml::{events::{BytesStart, Event}, Reader};

use serde_json::Value;
use tracing_subscriber::fmt::layer;

use crate::diplomacy::{map_parser::vector::{transform::Transform, utils::{get_id, get_json_string, SODIPODI_SIDES}}, persistence::{player::{Player, PlayerInfo}, province::{self, Coast, CoastReference, Province, ProvinceInfo, ProvinceReference, ProvinceType}, unit::{self, Unit, UnitType}}};

use super::utils::{get_attribute, get_inkspace_label, get_player, parse_path};



#[derive(PartialEq, Debug, Clone)]
enum Layer {
    None,
    LandLayer,
    IslandLayer,
    IslandFillLayer,
    SeaLayer,
    NamesLayer,
    CentersLayer,
    UnitsLayer
}

pub struct LayerInfo {
    land_layer: String,
    island_layer: String,
    island_fill_layer: String,
    sea_layer: String,
    names_layer: String,
    centers_layer: String,
    units_layer: Option<String>,
    // phantom_primary_armies_layer: String, 
    // phantom_retreat_armies_layer: String, 
    // phantom_primary_fleets_layer: String, 
    // phantom_retreat_fleets_layer: String, 
    province_labels: bool,
    unit_labels: bool,
    center_labels: bool,
    unit_type_labeled: bool,
    neutral: String,
    neutral_sc: String,
    border_margin_hint: f64,
    loc_x_offset: f64,
    loc_y_offset: f64,
}

impl LayerInfo {
    fn match_id(&self, id: Cow<'_, str>) -> Layer {
        if id == self.land_layer {
            Layer::LandLayer
        } else if id == self.island_layer {
            Layer::IslandLayer
        } else if id == self.island_fill_layer {
            Layer::IslandFillLayer
        } else if id == self.sea_layer {
            Layer::SeaLayer
        } else if id == self.names_layer {
            Layer::NamesLayer
        } else if id == self.centers_layer {
            Layer::CentersLayer
        } else if Some(id.to_string()) == self.units_layer {
            Layer::UnitsLayer
        } else {
            Layer::None
        }
    }
}

pub struct Parser {
    players: Vec<Arc<PlayerInfo>>,
    datafile: String,
    svg_path: PathBuf,
    color_to_player: HashMap<String, Option<Arc<PlayerInfo>>>,
    name_to_province: HashMap<String, Mutex<ProvinceInfo>>,
    layer_info: LayerInfo,
    // cache_provinces: Option<HashSet<Province>>,
    // cache_adjacencies: Option<HashSet<(String, String)>>
    overrides: Option<Value>
}

// requires reodering island adjacencies to occur before island fill shrug
impl Parser {
    pub fn new(data: String) -> Parser {
        let datafile = data;
        let current_dir = env::current_dir().unwrap();
        let data_path = current_dir.clone().join("config/".to_string() + &datafile); 
        println!("{:?}", data_path);

        let data = fs::read_to_string(data_path).expect("Failed to read file.");
        let json: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");

        let layers = json.get("svg config").expect("Expected \'svg config\' in json");
        let overrides = json.get("overrides").cloned();

        let svg_path = current_dir.clone().join(get_json_string(&json, "file"));

        let province_labels = layers.get("province_labels").map(|f| f.as_bool()).flatten().unwrap_or(false);
        let center_labels = layers.get("center_labels").map(|f| f.as_bool()).flatten().unwrap_or(false);
        let unit_labels = layers.get("unit_labels").map(|f| f.as_bool()).flatten().unwrap_or(false);
        let unit_type_labeled = layers.get("unit_type_labeled").map(|f| f.as_bool()).flatten().unwrap_or(false);

        let land_layer = get_json_string(layers, "land_layer").to_string();
        let island_layer = get_json_string(layers, "island_borders").to_string();
        let island_fill_layer = get_json_string(layers, "island_fill_layer").to_string();
        let sea_layer: String = get_json_string(layers, "sea_borders").to_string();
        let names_layer: String = get_json_string(layers, "province_names").to_string();
        let centers_layer = get_json_string(layers, "supply_center_icons").to_string();
        let units_layer = {
            if let Some(value) = layers.get("starting_units") {
                Some(value.as_str().unwrap().to_string())
            } else { 
                None
            }
        };

        let loc_x_offset = layers.get("loc_x_offset").and_then(|f| f.as_f64()).unwrap_or(0.0);
        let loc_y_offset = layers.get("loc_y_offset").and_then(|f| f.as_f64()).unwrap_or(0.0);
     
        let neutral = get_json_string(layers, "neutral").to_string();
        let neutral_sc = get_json_string(layers, "neutral_sc").to_string();
        let border_margin_hint = layers.get("border_margin_hint").unwrap().as_f64().unwrap();

        let mut players: Vec<Arc<PlayerInfo>> = Vec::new();
        let mut color_to_player: HashMap<String, Option<Arc<PlayerInfo>>> = HashMap::new();

        let players_json = json.get("players").expect("Expected \'players\' in json").as_object().expect("Players should be json object");

        for (name, data) in players_json {
            let color = data.get("color").expect("Player missing \'color\'").as_str().expect("Color should be string");
            let vscc = data.get("vscc").expect("Player missing \'vscc\'").as_i64().expect("Vscc should be int");
            let iscc = data.get("iscc").expect("Player missing \'iscc\'").as_i64().expect("Iscc should be int");
            let player = Arc::new(PlayerInfo { name: name.to_string(), color: color.to_string(), vscc, iscc });
            players.push(Arc::clone(&player));
            color_to_player.insert(color.to_string(), Some(player));
        }

        color_to_player.insert(neutral.clone(), None);
        color_to_player.insert(neutral_sc.clone(), None);

        Parser {
            players,
            datafile,
            svg_path,
            color_to_player,
            name_to_province: HashMap::new(),
            layer_info: LayerInfo {
                land_layer,
                island_layer,
                island_fill_layer,
                sea_layer,
                names_layer,
                centers_layer,
                units_layer,
                unit_type_labeled,
                province_labels,
                center_labels,
                unit_labels,
                neutral,
                neutral_sc,
                border_margin_hint,
                loc_x_offset,
                loc_y_offset
            },
            overrides
        }

        // let phantom_primary_armies_layer = get_svg_element(&svg_root, get_json_string(layers, "army")).unwrap();
        // let phantom_retreat_armies_layer = get_svg_element(&svg_root, get_json_string(layers, "retreat_army")).unwrap();
        // let phantom_primary_fleets_layer = get_svg_element(&svg_root, get_json_string(layers, "fleet")).unwrap();
        // let phantom_retreat_fleets_layer: Group = get_svg_element(&svg_root, get_json_string(layers, "retreat_fleet")).unwrap();
    }

    pub fn parse(&mut self) {
        let svg_str = fs::read_to_string(&self.svg_path).expect("Failed to read file.");
        let mut reader = Reader::from_str(svg_str.as_str());

        let mut depth = 0;
        let mut processed_layers: Vec<Layer> = Vec::new();
        let mut layer = Layer::None;

        // land layer, island layer, sea layer
        let mut raw_provinces: Option<Vec<ProvinceInfo>> = Some(Vec::new());
        let mut finished_provinces: bool = false;

        let mut province_layer_count = 0;

        // name layer

        loop {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    if depth == 1 {
                        println!("{:?}", get_id(&e));
                        layer = self.layer_info.match_id(get_id(&e));

                        match layer {
                            Layer::LandLayer => {
                                province_layer_count += 1;
                                self.parse_province_layer(ProvinceType::LAND, &mut reader, &mut raw_provinces, e)
                            },
                            Layer::IslandLayer => {
                                province_layer_count += 1;
                                self.parse_province_layer(ProvinceType::ISLAND, &mut reader, &mut raw_provinces, e)
                            },
                            Layer::SeaLayer => {
                                province_layer_count += 1;
                                self.parse_province_layer(ProvinceType::SEA, &mut reader, &mut raw_provinces, e)
                            },
                            Layer::NamesLayer => {
                                if self.layer_info.province_labels {
                                    depth += 1
                                } else {
                                    todo!("support names layer")
                                }
                            },
                            Layer::CentersLayer => {
                                if self.layer_info.center_labels {
                                    self.parse_supply_centers_assisted(&mut reader)
                                } else {
                                    todo!("support centers layer")
                                }
                            },
                            Layer::UnitsLayer => {
                                if self.layer_info.unit_labels {
                                    self.parse_units_assisted(&mut reader)
                                } else {
                                    todo!("support units layer")
                                }
                            },

                            Layer::IslandFillLayer => {
                                self.parse_island_fill_layer(&mut reader);
                            },
                            Layer::None => depth += 1,

                        }
                    } else {
                        depth += 1;
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                    
                    if processed_layers.contains(&layer) {
                        panic!("Layer {:?} processed twice", layer);
                    }

                    if layer != Layer::None {
                        processed_layers.push(layer.clone()); 
                    }

                    layer = Layer::None;

                    if province_layer_count == 3 && !finished_provinces {
                        finished_provinces = true;
                        self.process_provinces(raw_provinces.take().unwrap());
                        println!("done processing provinces ");
                    }
                },
                _ => {}
            }
        }

        //         for province in provinces:
        //             province.all_locs -= {None}
        //             province.all_rets -= {None}
        //             if province.primary_unit_coordinate == None:
        //                 logger.warning(f"Province {province.name} has no unit coord. Setting to 0,0 ...")
        //                 province.primary_unit_coordinate = (0, 0)
        //             if province.retreat_unit_coordinate == None:
        //                 logger.warning(f"Province {province.name} has no retreat coord. Setting to 0,0 ...")
        //                 province.retreat_unit_coordinate = (0, 0)

        //         for province in provinces:
        //             for coast in province.coasts:
        //                 coast.all_locs -= {None}
        //                 coast.all_rets -= {None}
        //                 if coast.primary_unit_coordinate == None:
        //                     logger.warning(f"Province {coast.name} has no unit coord. Setting to 0,0 ...")
        //                     coast.primary_unit_coordinate = (0, 0)
        //                 if coast.retreat_unit_coordinate == None:
        //                     logger.warning(f"Province {coast.name} has no retreat coord. Setting to 0,0 ...")
        //                     coast.retreat_unit_coordinate = (0, 0)


        //         return Board(players, provinces, units, phase.initial(), self.data, self.datafile)

        // todo create provinces array index and reference using index

        for province in self.name_to_province.values() {
            println!("{:?}", province);
        }

    }

    pub fn parse_province_layer(&mut self, province_type: ProvinceType, reader: &mut Reader<&[u8]>, raw_provinces: &mut Option<Vec<ProvinceInfo>>, e: BytesStart) {
        let mut depth = 0;

        let layer_transform = Transform::get_transform(&e);

        // name layer

        while depth >= 0 {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(_)) => {
                    depth += 1;
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                },
                Ok(Event::Empty(e)) => {
                    // create provinces
                    let mut province = self.parse_province(&e, province_type.clone(), &layer_transform);
                    // set owner for island layer
                    if province_type == ProvinceType::LAND {
                        province.initial_owner = get_player(&e, &self.color_to_player);
                    }

                    raw_provinces.as_mut().unwrap().push(province);
                },
                _ => {}
            }
        }
    }

    pub fn parse_island_fill_layer(&mut self, reader: &mut Reader<&[u8]>) {
        let mut depth = 0;

        while depth >= 0 {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(_)) => {
                    depth += 1;
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                },
                Ok(Event::Empty(e)) => {
                    let name = get_inkspace_label(&e);

                    let province = self.name_to_province.get_mut(&name.into_owned()).expect("Unknown Province Name");

                    province.try_lock().unwrap().initial_owner = get_player(&e, &self.color_to_player);
                },
                _ => {}
            }
        }
    }

    pub fn parse_supply_centers_assisted(&mut self, reader: &mut Reader<&[u8]>) {
        let mut depth = 0;

        while depth >= 0 {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth == 1 {
                        let name = get_inkspace_label(&e).into_owned();

                        let mut province = self.name_to_province.get_mut(&name).expect(format!("Unknown Province {}", name).as_str()).try_lock().unwrap();

                        if province.has_supply_center {
                            panic!("{} already has a supply center", name)
                        }

                        province.has_supply_center = true;
                        
                        province.initial_core = province.initial_owner.clone();
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                },
                _ => {}
            }
        }
    }

    pub fn parse_units_assisted(&mut self, reader: &mut Reader<&[u8]>) {
        let mut depth = 0;

        let mut unit_type = UnitType::ARMY;
        let mut name = String::from("");

        while depth >= 0 {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth == 1 {
                        name = get_inkspace_label(&e).into_owned();
                    }

                    // if get
                }
                Ok(Event::End(_)) => {
                    if depth == 1 {
                        if self.layer_info.unit_type_labeled {
                            let first_char = name.chars().next().unwrap().to_ascii_lowercase();
                            unit_type = match first_char {
                                'f' => UnitType::FLEET,
                                'a' => UnitType::ARMY,
                                _ => panic!("Unit types are labeled, but {} doesn't start with F or A", name)
                            };
                            name = name.chars().skip(1).collect();
                        }
    
                        let (province, coast) = self.get_province_and_coast(name.clone());
                        let mut province = province.try_lock().unwrap();
                        let player = province.initial_owner.as_ref().unwrap().clone();

                        let coast = if let Some(coast) = coast {
                            Some(coast)
                        } else {
                            province.coasts.as_ref().map(|f| f.iter().next()).flatten()
                                .map(|f| f.to_reference())
                        };

                        // println!("_____________________");
                        // println!("{}", name);
                        // println!("{:?}", player);
                        // println!("{:?}", unit_type);
                        // println!("_____________________");

                        province.initial_unit = Some(Unit {
                            unit_type: unit_type.clone(),
                            owner: player,
                            current_province: province.to_reference(),
                            coast,
                            retreat_options: None,
                        });
                    }

                    depth -= 1;
                },
                Ok(Event::Empty(e)) => {
                    if e.local_name().as_ref() == b"path" {
                        // println!("{:?}", e);
                        let num_sides = get_attribute(&e, SODIPODI_SIDES).unwrap().into_owned();
                        let num_sides = num_sides.as_str();
                        unit_type = {
                            match num_sides {
                                "3" => UnitType::FLEET,
                                "6" => UnitType::ARMY,
                                _ => panic!("Unit has {} sides which does not match any unit definition.", num_sides)
                            }
                        };
                    }
                },
                _ => {}
            }
        }
    }

    pub fn get_province_and_coast(&mut self, mut province_name: String) -> (&Mutex<ProvinceInfo>, Option<CoastReference>) {
        let mut coast_suffix = None;
        let coast_names = [" (nc)", " (sc)", " (ec)", " (wc)"];

        for coast_name in coast_names {
            if province_name.ends_with(coast_name) {
                province_name.truncate(province_name.len() - 5);

                coast_suffix = Some(&coast_name[2..4]);
                break
            }
        }

        let province = self.name_to_province.get(province_name.as_str()).unwrap();
        let mut coast = None;
        if let Some(suffix) = coast_suffix {
            if let Some(coasts) = &province.try_lock().unwrap().coasts {
                for potential in coasts {
                    if potential.name == format!("{} {}", province_name, suffix) {
                        coast = Some(potential.to_reference())
                    }
                }
            }
        }

        return (province, coast)
    }

    pub fn process_provinces(&mut self, mut provinces: Vec<ProvinceInfo>) {
        let adjacencies_path = env::current_dir().unwrap().clone().join(format!("config/{}_adjacencies.txt", self.datafile).as_str()); 

        let mut adjacencies: Vec<(String, String)> = Vec::new();

        if fs::exists(&adjacencies_path).unwrap() {
            let file = File::open(&adjacencies_path).unwrap();
            let mut lines = BufReader::new(file).lines();
            while let Some(Ok(line)) = lines.next() {
                // println!("{}", line);
                let (a, b) = line.split_once(',').unwrap();
                adjacencies.push((a.to_string(), b.to_string()));
            }
        } else {
            let file = File::create(&adjacencies_path).unwrap();
            let mut writer = BufWriter::new(&file);

            for i in 0..provinces.len() {
                let (left, right) = provinces.split_at_mut(i+1);
                let a = &mut left[i];

                for b in right {
                    if a.geometry.as_ref().unwrap().distance(b.geometry.as_ref().unwrap()).unwrap() < self.layer_info.border_margin_hint {
                        writeln!(&mut writer, "{},{}", a.name, b.name).unwrap();
                        adjacencies.push((a.name.clone(), b.name.clone()));
                    }
                }
            }
        }

        for province in provinces {
            let name = &province.name;
            if self.name_to_province.contains_key(name) {
                panic!("{} repeats in map,", province.name)
            }

            self.name_to_province.insert(name.to_string(), Mutex::new(province));
        }

        for (name1, name2) in adjacencies {
            if name1 == name2 {
                panic!("Adjacency has two of the same values")
            }

            let mut a = self.name_to_province.get(&name1).unwrap().try_lock().unwrap();
            let mut b = self.name_to_province.get(&name2).unwrap().try_lock().unwrap();
            a.adjacent.push(b.to_reference());
            b.adjacent.push(a.to_reference());
        }

        self.json_cheats();

        for province in self.name_to_province.values() {
            province.try_lock().unwrap().set_coasts(|p| self.resolve_reference(p))
        }
//         # set phantom unit coordinates for optimal unit placements
//         self._set_phantom_unit_coordinates()

//         for province in provinces:
//             province.all_locs.add(province.primary_unit_coordinate)
//             province.all_rets.add(province.retreat_unit_coordinate)
//             for coast in province.coasts:
//                 coast.all_locs.add(coast.primary_unit_coordinate)
//                 coast.all_rets.add(coast.retreat_unit_coordinate)

//         return provinces
    }

    fn resolve_reference(&self, p: &ProvinceReference) -> &Mutex<ProvinceInfo> {
        match p {
            ProvinceReference::Name(name) => {
                self.name_to_province.get(name).unwrap()
            },
            ProvinceReference::Index(_) => panic!("Shouldn't occur"),
        }
    }

    fn json_cheats(&mut self) {
        if self.overrides.is_none() {
            return;
        }

        let overrides = self.overrides.take().unwrap();
        
        println!("fooy");
        // println!("{:?}", overrides);

        if let Some(high_provinces) = overrides.get("high provinces").and_then(|f| f.as_object()) {
            for (name, data) in high_provinces {
                let num: i64 = data.get("num").unwrap().as_i64().unwrap();
                let province_type = match data.get("type").unwrap().as_str().unwrap() {
                    "SEA" => ProvinceType::SEA,
                    "LAND" => ProvinceType::LAND,
                    _ => panic!("Unknown high province type")
                };
                let mut high_provinces = Vec::new();

                for index in 1..=num {
                    let high_name = name.to_owned() + &index.to_string();
                    let high_province: ProvinceInfo = ProvinceInfo {
                        name: high_name.clone(),
                        province_type: province_type.clone(),
                        adjacent: Vec::new(),
                        has_supply_center: false,
                        initial_owner: None,
                        initial_core: None,
                        initial_unit: None,
                        geometry: None,
                        coasts: None,
                    };

                    high_provinces.push(high_province.to_reference());

                    if self.name_to_province.contains_key(&high_name) {
                        panic!("{} repeats in map,", high_name);
                    }
                    self.name_to_province.insert(high_name, Mutex::new(high_province));
                }
                // println!("{:?}", data);
            }

            for (name, data) in high_provinces {
                let num: i64 = data.get("num").unwrap().as_i64().unwrap();
                let adjacent_provinces: Vec<ProvinceReference> = data.get("adjacencies").unwrap().as_array().unwrap()
                    .iter().map(|f| f.as_str().unwrap())
                    .map(|f| {
                        if let Some(p) = self.name_to_province.get(f) {
                            p.try_lock().unwrap().to_reference()
                        } else {
                            panic!("Unknown key {}", f)
                        }
                    }).collect();
                let mut high_provinces = Vec::new();
                for index in 1..=num {
                    let high_name = name.to_owned() + &index.to_string();

                    high_provinces.push(self.name_to_province.get(&high_name).unwrap().try_lock().unwrap().to_reference());
                }

                // println!("{:?}", data);

                // add adjacencies of high provinces
                for high_province in &high_provinces {
                    for adjacent_province in &adjacent_provinces {
                        let mut a = self.resolve_reference(high_province).try_lock().unwrap();
                        let mut b = self.resolve_reference(adjacent_province).try_lock().unwrap();

                        a.adjacent.push(b.to_reference());
                        b.adjacent.push(a.to_reference());
                    }
                }

                // high provinces are mutually adjacent
                for a in &high_provinces {
                    for b in &high_provinces {
                        if a != b {
                            self.resolve_reference(a).try_lock().unwrap().adjacent.push(b.clone());
                        }
                    }
                }
            }
        }

        if let Some(provinces) = overrides.get("provinces").and_then(|f| f.as_object()) {
            for (name, data) in provinces {
                let province = self.name_to_province.get(name).unwrap();
                println!("{:?}", name);
                println!("{:?}", data);

                if let Some(adjacencies) = data.get("adjacencies").and_then(|f| f.as_array()) {
                    todo!();
                }
                if let Some(remove_adjacencies) = data.get("remove_adjacencies").and_then(|f| f.as_array()) {
                    todo!();
                }
                if let Some(coasts) = data.get("coasts").and_then(|f| f.as_object()) {
                    todo!();
                }
                if let Some(unit_locs) = data.get("unit_loc").and_then(|f| f.as_array()) {
                    todo!();
                    println!("{:?}", unit_locs);
                    for coordinate in unit_locs {

                    }

//                     for coordinate in data["unit_loc"]:
//                         coordinate = tuple((tuple(coordinate) + offset).tolist())
//                         province.all_locs.add(coordinate)
//                         province.primary_unit_coordinate = coordinate

                    
                }
                if let Some(retreat_unit_locs) = data.get("unit_loc").and_then(|f| f.as_array()) {
                    todo!();
                }

            }
        }



//         if "provinces" in self.data["overrides"]:
//             for name, data in self.data["overrides"]["provinces"].items():
//                 province = self.name_to_province[name]
//                 # TODO: Some way to specifiy whether or not to clear other adjacencies?
//                 if "adjacencies" in data:
//                     province.adjacent.update(self.names_to_provinces(data["adjacencies"]))
//                 if "remove_adjacencies" in data:
//                     province.adjacent.difference_update(self.names_to_provinces(data["remove_adjacencies"]))
//                 if "coasts" in data:
//                     province.coasts = set()
//                     for coast_name, coast_adjacent in data["coasts"].items():
//                         coast = Coast(f"{name} {coast_name}", None, None, set(self.names_to_provinces(coast_adjacent)), province)
//                         province.coasts.add(coast)
//                 if "unit_loc" in data:
//                     for coordinate in data["unit_loc"]:
//                         coordinate = tuple((tuple(coordinate) + offset).tolist())
//                         province.all_locs.add(coordinate)
//                         province.primary_unit_coordinate = coordinate
//                 if "retreat_unit_loc" in data:
//                     for coordinate in data["retreat_unit_loc"]:
//                         coordinate = tuple((tuple(coordinate) + offset).tolist())
//                         province.all_rets.add(coordinate)
//                         province.retreat_unit_coordinate = coordinate

//         return provinces
        
    }

    fn parse_province(
        &self,
        province_data: &BytesStart<'_>,
        province_type: ProvinceType,
        layer_translation: &Transform
    ) -> ProvinceInfo {
        if province_data.local_name().as_ref() != b"path" {
            panic!("Province border has non path attribute");
        }

        let path_string = get_attribute(&province_data, "d").expect("Province path data not found");

            // TODO all this
        let this_translation = Transform::get_transform(&province_data);

        let province_coordinates = parse_path(&path_string, layer_translation, &this_translation);

        let name = {
            if self.layer_info.province_labels {
                get_inkspace_label(&province_data).into_owned()
            } else {
                String::from("")
            }
        };
        
        let polygons: Vec<Geometry> = province_coordinates.into_iter().map(|p| -> Geometry {
            Geometry::try_from(p).expect("Failed Conversion")
        }).collect();

        let multi = polygons.len() > 1;


        let mut geometry = Geometry::create_geometry_collection(polygons).unwrap();
        
        if multi {
            geometry = geometry.buffer(0.1, 16).unwrap();
        }

        ProvinceInfo {
            name,
            province_type,
            geometry: Some(geometry),
            adjacent: Vec::new(),
            has_supply_center: false,
            initial_owner: None,
            initial_core: None,
            initial_unit: None,
            coasts: None,
        }
    }
}


//     def names_to_provinces(self, names: set[str]):
//         return map((lambda n: self.name_to_province[n]), names)

//     def add_province_to_board(self, provinces: set[Province], province: Province) -> set[Province]:
//         provinces = {x for x in provinces if x.name != province.name}
//         provinces.add(province)
//         self.name_to_province[province.name] = province
//         return provinces

//     def _set_phantom_unit_coordinates(self) -> None:
//         army_layer_to_key = [
//             (self.phantom_primary_armies_layer, "primary_unit_coordinate"),
//             (self.phantom_retreat_armies_layer, "retreat_unit_coordinate"),
//         ]
//         for layer, province_key in army_layer_to_key:
//             layer_translation = get_transform(layer)
//             for unit_data in layer.getchildren():
//                 unit_translation = get_transform(unit_data)
//                 province = self._get_province(unit_data)
//                 coordinate = get_unit_coordinates(unit_data)
//                 setattr(province, province_key, layer_translation.transform(unit_translation.transform(coordinate)))

//         fleet_layer_to_key = [
//             (self.phantom_primary_fleets_layer, "primary_unit_coordinate"),
//             (self.phantom_retreat_fleets_layer, "retreat_unit_coordinate"),
//         ]
//         for layer, province_key in fleet_layer_to_key:

//             layer_translation = get_transform(layer)
//             for unit_data in layer.getchildren():
//                 unit_translation = get_transform(unit_data)
//                 # This could either be a sea province or a land coast
//                 province_name = self._get_province_name(unit_data)
//                 # this is me writing bad code to get this out faster, will fix later when we clean up this file
//                 province, coast = self._get_province_and_coast(province_name)
//                 is_coastal = False
//                 for adjacent in province.adjacent:
//                     if adjacent.type != ProvinceType.LAND:
//                         is_coastal = True
//                         break
//                 if not coast and province.type != ProvinceType.SEA and is_coastal:
//                     # bad bandaid: this is probably an extra phantom unit, or maybe it's a primary one?
//                     try:
//                         coast = province.coast()
//                     except Exception:
//                         print(
//                             f"Warning: phantom unit skipped, if drawing some move doesn't work this might be why: {province_name} {province_key}"
//                         )
//                         continue

//                 coordinate = get_unit_coordinates(unit_data)
//                 translated_coordinate = unit_translation.transform(layer_translation.transform(coordinate))
//                 if coast:
//                     setattr(coast, province_key, translated_coordinate)
//                 else:
//                     setattr(province, province_key, translated_coordinate)

//     def _get_province_name(self, province_data: Element) -> str:
//         return province_data.get(f"{NAMESPACE.get('inkscape')}label")

//     def _get_province(self, province_data: Element) -> Province:
//         return self.name_to_province[self._get_province_name(province_data)]