
use std::{borrow::Cow, cell::RefCell, collections::HashMap, env, fs::{self, File}, io::{BufRead, BufReader, BufWriter, Write}, path::PathBuf, sync::Arc};

use geo_types::Coord;
use geos::{Geom, Geometry};
use quick_xml::{events::{BytesStart, Event}, Reader};

use serde_json::Value;

use crate::diplomacy::{config_parser::config::LayerInfo, map_parser::vector::{transform::Transform, utils::{get_id, get_json_string, SODIPODI_SIDES}}, persistence::{board::BoardInfo, player::PlayerInfo, province::{Coast, CoastReference, Coords, ProvinceInfo, ProvinceReference, ProvinceType}, unit::{Unit, UnitType}}};

use super::utils::{get_attribute, get_inkspace_label, get_player, get_unit_coordinates, parse_path};



#[derive(PartialEq, Debug, Clone)]
enum Layer {
    None,
    LandLayer,
    IslandLayer,
    IslandFillLayer,
    SeaLayer,
    NamesLayer,
    CentersLayer,
    UnitsLayer,
    PhantomPrimaryArmiesLayer,
    PhantomRetreatArmiesLayer,
    PhantomPrimaryFleetsLayer,
    PhantomRetreatFleetsLayer
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
        } else if id == self.phantom_primary_armies_layer {
            Layer::PhantomPrimaryArmiesLayer
        } else if id == self.phantom_retreat_armies_layer {
            Layer::PhantomRetreatArmiesLayer
        } else if id == self.phantom_primary_fleets_layer {
            Layer::PhantomPrimaryFleetsLayer
        } else if id == self.phantom_retreat_fleets_layer {
            Layer::PhantomRetreatFleetsLayer
        }else if Some(id.to_string()) == self.units_layer {
            Layer::UnitsLayer
        } else {
            Layer::None
        }
    }
}

pub struct Parser {
    name: String,
    players: Vec<Arc<PlayerInfo>>,
    datafile: String,
    svg_path: PathBuf,
    color_to_player: HashMap<String, Option<Arc<PlayerInfo>>>,
    name_to_province: HashMap<String, RefCell<ProvinceInfo>>,
    layer_info: LayerInfo,
    overrides: Option<Value>
}

// requires reordering island adjacencies to occur before island fill shrug if that is ever used for names
impl Parser {
    pub fn new(data: String) -> Parser {
        let datafile = data;
        let current_dir = env::current_dir().unwrap();
        let data_path = current_dir.clone().join("config/".to_string() + &datafile); 
        println!("{:?}", data_path);

        let data = fs::read_to_string(data_path).expect("Failed to read file.");
        let json: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
        let name = json.get("name").unwrap().as_str().unwrap().to_string();
        let overrides = json.get("overrides").cloned();

        let layers: &Value = json.get("svg config").expect("Expected \'svg config\' in json");
        let layer_info = LayerInfo::new(layers);

        let svg_path = current_dir.clone().join(get_json_string(&json, "file"));

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

        color_to_player.insert(layer_info.neutral.clone(), None);
        color_to_player.insert(layer_info.neutral_sc.clone(), None);

        Parser {
            name,
            players,
            datafile,
            svg_path,
            color_to_player,
            name_to_province: HashMap::new(),
            layer_info,
            overrides
        }

    }

    pub fn parse(mut self) -> BoardInfo {
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
                        // println!("{:?}", get_id(&e));
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
                            Layer::PhantomPrimaryArmiesLayer => {
                                self.parse_phantom_army_layer(&mut reader, e, false);
                            },
                            Layer::PhantomRetreatArmiesLayer => {
                                self.parse_phantom_army_layer(&mut reader, e, true);
                            },
                            Layer::PhantomPrimaryFleetsLayer => {
                                self.parse_phantom_fleet_layer(&mut reader, e, false);
                            },
                            Layer::PhantomRetreatFleetsLayer => {
                                self.parse_phantom_fleet_layer(&mut reader, e, true);
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

        for (name, province) in self.name_to_province.iter_mut() {
            let mut province = province.borrow_mut();
            let coords = &mut province.coords;

            if coords.primary_unit_coordinate == None {
                println!("Province {} has no unit coord. Setting to 0,0 ...", name);
                //                 logger.warning(f"Province {province.name} has no unit coord. Setting to 0,0 ...")
                coords.primary_unit_coordinate = Some(Coord { x: 0.0, y: 0.0 });
            }
            if coords.retreat_unit_coordinate == None {
                println!("Province {} has no retreat coord. Setting to 0,0 ...", name);
                //                 logger.warning(f"Province {province.name} has no retreat coord. Setting to 0,0 ...")
                coords.retreat_unit_coordinate = Some(Coord { x: 0.0, y: 0.0 });
            }

            if let Some(coasts) = &mut province.coasts {
                for coast in coasts {
                    let name = coast.name.clone();
                    let coords = &mut coast.coords;
                    if coords.primary_unit_coordinate == None {
                        println!("Province {} has no unit coord. Setting to 0,0 ...", name);
                        //                 logger.warning(f"Province {province.name} has no unit coord. Setting to 0,0 ...")
                        coords.primary_unit_coordinate = Some(Coord { x: 0.0, y: 0.0 });
                    }
                    if coords.retreat_unit_coordinate == None {
                        println!("Province {} has no retreat coord. Setting to 0,0 ...", name);
                        //                 logger.warning(f"Province {province.name} has no retreat coord. Setting to 0,0 ...")
                        coords.retreat_unit_coordinate = Some(Coord { x: 0.0, y: 0.0 });
                    }
                }
            }
        }

        let name_to_info = self.name_to_province.into_iter().map(|(k, v)| {
            (k, Arc::new(v.into_inner()))
        }).collect();

        BoardInfo {
            name: self.name,
            players: self.players,
            name_to_info,
            datafile: self.datafile
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

                    province.borrow_mut().initial_owner = get_player(&e, &self.color_to_player);
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

                        let mut province = self.name_to_province.get_mut(&name).expect(format!("Unknown Province {}", name).as_str()).borrow_mut();

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
        let mut depth: i32 = 0;

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
                        let mut province = province.borrow_mut();
                        let player = province.initial_owner.as_ref().unwrap().clone();

                        let coast = if let Some(coast) = coast {
                            Some(coast)
                        } else {
                            province.coasts.as_ref().map(|f| f.iter().next()).flatten()
                                .map(|f| f.to_reference())
                        };

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

    fn parse_phantom_army_layer(&mut self, reader: &mut Reader<&[u8]>, e: BytesStart, retreat: bool) {
        let mut depth: i32 = 0;

        let layer_transform = Transform::get_transform(&e);
        let mut unit_translation = Transform::empty();
        let mut name = String::from("");
        let mut coord = Coord { x: 0.0, y: 0.0};

        while depth >= 0 {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth == 1 {
                        unit_translation = Transform::get_transform(&e);
                        name = get_inkspace_label(&e).into_owned();
                    }
                }
                Ok(Event::End(_)) => {
                    if depth == 1 {
                        let mut province = self.name_to_province.get(&name).unwrap().borrow_mut();
                        let coord = Some(layer_transform.transform(unit_translation.transform(coord)));
                        if retreat {
                            province.coords.retreat_unit_coordinate = coord;
                        } else {
                            province.coords.primary_unit_coordinate = coord;
                        }
                    }
                    depth -= 1;
                },
                Ok(Event::Empty(e)) => {
                    if depth == 0 {
                        panic!("Unknown thing to handle");
                    }

                    if e.local_name().as_ref() == b"path" {
                        coord = get_unit_coordinates(&e);
                    }
                },
                _ => {}
            }
        }
    }

    fn parse_phantom_fleet_layer(&mut self, reader: &mut Reader<&[u8]>, e: BytesStart, retreat: bool) {
        let mut depth: i32 = 0;

        let layer_transform = Transform::get_transform(&e);
        let mut unit_translation = Transform::empty();
        let mut name = String::from("");
        let mut coord = Coord { x: 0.0, y: 0.0};

        while depth >= 0 {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e), // TODO proper
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth == 1 {
                        unit_translation = Transform::get_transform(&e);
                        name = get_inkspace_label(&e).into_owned();
                    }
                }
                Ok(Event::End(_)) => {
                    if depth == 1 {
                        let (province_cell, mut coast) = self.get_province_and_coast(name.clone());
                    
                        let mut province = province_cell.borrow_mut();
                        if coast.is_none() && province.province_type != ProvinceType::SEA {
                            let is_coastal = province.adjacent.iter()
                            .any(|f| self.resolve_reference(f).borrow().province_type != ProvinceType::LAND);
                            // println!("{}", name);
                            // println!("{}", is_coastal);
                            if is_coastal {
                                if let Some(c) = province.coasts.as_ref().and_then(|f| f.iter().next()) {
                                    coast = Some(c.to_reference());
                                } else {
                                    panic!("Warning: phantom unit skipped, if drawing some move doesn't work this might be why: {}", name);
                                }
                            }
                        }

                        let coord = Some(layer_transform.transform(unit_translation.transform(coord)));
                        
                        if retreat {
                            if let Some(coast) = coast {
                                province.resolve_reference(coast).coords.primary_unit_coordinate = coord;
                            } else {
                                province.coords.primary_unit_coordinate = coord;
                            }
                        } else {
                            if let Some(coast) = coast {
                                province.resolve_reference(coast).coords.retreat_unit_coordinate = coord;
                            } else {
                                province.coords.retreat_unit_coordinate = coord;
                            }
                        }
                    }

                    depth -= 1;
                },
                Ok(Event::Empty(e)) => {
                    if depth == 0 {
                        panic!("Unknown thing to handle");
                    }

                    if e.local_name().as_ref() == b"path" {
                        coord = get_unit_coordinates(&e);
                    }
                },
                _ => {}
            }
        }
    }

    pub fn get_province_and_coast(&self, mut province_name: String) -> (&RefCell<ProvinceInfo>, Option<CoastReference>) {
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
            if let Some(coasts) = &province.borrow().coasts {
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

            self.name_to_province.insert(name.to_string(), RefCell::new(province));
        }

        for (name1, name2) in adjacencies {
            if name1 == name2 {
                panic!("Adjacency has two of the same values")
            }

            let mut a = self.name_to_province.get(&name1).unwrap().borrow_mut();
            let mut b = self.name_to_province.get(&name2).unwrap().borrow_mut();
            a.adjacent.push(b.to_reference());
            b.adjacent.push(a.to_reference());
        }

        self.json_cheats();

        for province in self.name_to_province.values() {
            province.borrow_mut().set_coasts(|p| self.resolve_reference(p))
        }
    }

    fn resolve_reference(&self, p: &ProvinceReference) -> &RefCell<ProvinceInfo> {
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
                        coords: Coords {
                            all_locs: Vec::new(),
                            all_rets: Vec::new(),
                            primary_unit_coordinate: None,
                            retreat_unit_coordinate: None,
                        }
                    };

                    high_provinces.push(high_province.to_reference());

                    if self.name_to_province.contains_key(&high_name) {
                        panic!("{} repeats in map,", high_name);
                    }
                    self.name_to_province.insert(high_name, RefCell::new(high_province));
                }
                // println!("{:?}", data);
            }

            for (name, data) in high_provinces {
                let num: i64 = data.get("num").unwrap().as_i64().unwrap();
                let adjacent_provinces: Vec<ProvinceReference> = data.get("adjacencies").unwrap().as_array().unwrap()
                    .iter().map(|f| f.as_str().unwrap())
                    .map(|f| {
                        if let Some(p) = self.name_to_province.get(f) {
                            p.borrow().to_reference()
                        } else {
                            panic!("Unknown key {}", f)
                        }
                    }).collect();
                let mut high_provinces = Vec::new();
                for index in 1..=num {
                    let high_name = name.to_owned() + &index.to_string();

                    high_provinces.push(self.name_to_province.get(&high_name).unwrap().borrow().to_reference());
                }

                // add adjacencies of high provinces
                for high_province in &high_provinces {
                    for adjacent_province in &adjacent_provinces {
                        let mut a = self.resolve_reference(high_province).borrow_mut();
                        let mut b = self.resolve_reference(adjacent_province).borrow_mut();

                        a.adjacent.push(b.to_reference());
                        b.adjacent.push(a.to_reference());
                    }
                }

                // high provinces are mutually adjacent
                for a in &high_provinces {
                    for b in &high_provinces {
                        if a != b {
                            self.resolve_reference(a).borrow_mut().adjacent.push(b.clone());
                        }
                    }
                }
            }
        }

        if let Some(provinces) = overrides.get("provinces").and_then(|f| f.as_object()) {
            for (name, data) in provinces {
                let mut province = self.name_to_province.get(name).unwrap().borrow_mut();
                println!("{:?}", name);
                println!("{:?}", data);

                if let Some(adjacencies) = data.get("adjacencies").and_then(|f| f.as_array()) {
                    let mut to_add: Vec<ProvinceReference> = adjacencies.iter()
                        .map(|f| f.as_str().unwrap()).map(|f| {
                            if let Some(p) = self.name_to_province.get(f) {
                                p.borrow().to_reference()
                            } else {
                                panic!("Unknown key {}", f)
                            }        
                        }).collect();
                    province.adjacent.append(&mut to_add);
                }
                if let Some(remove_adjacencies) = data.get("remove_adjacencies").and_then(|f| f.as_array()) {
                    let to_remove: Vec<ProvinceReference> = remove_adjacencies.iter()
                        .map(|f| f.as_str().unwrap()).map(|f| {
                            if let Some(p) = self.name_to_province.get(f) {
                                p.borrow().to_reference()
                            } else {
                                panic!("Unknown key {}", f)
                            }        
                        }).collect();
                    province.adjacent = province.adjacent.iter().filter(|f| !to_remove.contains(f))
                        .map(|f| f.clone()).collect();
                }
                if let Some(coasts_map) = data.get("coasts").and_then(|f| f.as_object()) {
                    let mut coasts = Vec::new();
                    for (coast_name, coast_adjacent) in coasts_map {
                        let adjacent_seas: Vec<ProvinceReference> = coast_adjacent.as_array().unwrap()
                            .iter().map(|f| f.as_str().unwrap()).map(|f| {
                                if let Some(p) = self.name_to_province.get(f) {
                                    p.borrow().to_reference()
                                } else {
                                    panic!("Unknown key {}", f)
                                }        
                            }).collect();
                        let coast = Coast {
                            name: format!("{} {}", name, coast_name),
                            adjacent_seas,
                            province: province.to_reference(),
                            coords: Coords { 
                                all_locs: Vec::new(), 
                                all_rets: Vec::new(),
                                primary_unit_coordinate: None, 
                                retreat_unit_coordinate: None
                            }
                        };
                        coasts.push(coast);
                    }

                    province.coasts = Some(coasts);
                }
                if let Some(unit_locs) = data.get("unit_loc").and_then(|f| f.as_array()) {
                    for coordinate in unit_locs {
                        let a = coordinate.as_array().unwrap();
                        let x = a[0].as_f64().unwrap();
                        let y = a[1].as_f64().unwrap();
                        let coord = Coord { x: x + self.layer_info.loc_x_offset , y: y + self.layer_info.loc_y_offset };
                        province.coords.all_locs.push(coord.clone());
                        province.coords.primary_unit_coordinate = Some(coord);
                    }                    
                }
                if let Some(retreat_unit_locs) = data.get("unit_loc").and_then(|f| f.as_array()) {
                    for coordinate in retreat_unit_locs {
                        let a = coordinate.as_array().unwrap();
                        let x = a[0].as_f64().unwrap();
                        let y = a[1].as_f64().unwrap();
                        let coord = Coord { x: x + self.layer_info.loc_x_offset , y: y + self.layer_info.loc_y_offset };
                        province.coords.all_rets.push(coord.clone());
                        province.coords.retreat_unit_coordinate = Some(coord);
                    }                    
                }
            }
        }
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
            coords: Coords { 
                all_locs: Vec::new(), 
                all_rets: Vec::new(),
                primary_unit_coordinate: None, 
                retreat_unit_coordinate: None
            }
        }
    }

}