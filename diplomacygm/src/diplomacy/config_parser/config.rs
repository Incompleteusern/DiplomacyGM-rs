use serenity::json::Value;

use crate::diplomacy::map_parser::vector::utils::get_json_string;

pub struct LayerInfo {
    pub land_layer: String,
    pub island_layer: String,
    pub island_fill_layer: String,
    pub sea_layer: String,
    pub names_layer: String,
    pub centers_layer: String,
    pub units_layer: Option<String>,
    pub phantom_primary_armies_layer: String, 
    pub phantom_retreat_armies_layer: String, 
    pub phantom_primary_fleets_layer: String, 
    pub phantom_retreat_fleets_layer: String, 
    pub province_labels: bool,
    pub unit_labels: bool,
    pub center_labels: bool,
    pub unit_type_labeled: bool,
    pub neutral: String,
    pub neutral_sc: String,
    pub border_margin_hint: f64,
    pub loc_x_offset: f64,
    pub loc_y_offset: f64,
}


impl LayerInfo {
    pub fn new(layers: &Value) -> LayerInfo {

        let province_labels = layers.get("province_labels").and_then(|f| f.as_bool()).unwrap_or(false);
        let center_labels = layers.get("center_labels").and_then(|f| f.as_bool()).unwrap_or(false);
        let unit_labels = layers.get("unit_labels").and_then(|f| f.as_bool()).unwrap_or(false);
        let unit_type_labeled = layers.get("unit_type_labeled").and_then(|f| f.as_bool()).unwrap_or(false);

        let land_layer = get_json_string(layers, "land_layer").to_string();
        let island_layer = get_json_string(layers, "island_borders").to_string();
        let island_fill_layer = get_json_string(layers, "island_fill_layer").to_string();
        let sea_layer: String = get_json_string(layers, "sea_borders").to_string();
        let names_layer: String = get_json_string(layers, "province_names").to_string();
        let centers_layer = get_json_string(layers, "supply_center_icons").to_string();
        let units_layer = {
            layers.get("starting_units").map(|value| value.as_str().unwrap().to_string())
        };
        let phantom_primary_armies_layer = get_json_string(layers, "army").to_string();
        let phantom_retreat_armies_layer = get_json_string(layers, "retreat_army").to_string();
        let phantom_primary_fleets_layer = get_json_string(layers, "fleet").to_string();
        let phantom_retreat_fleets_layer = get_json_string(layers, "retreat_fleet").to_string();

        let loc_x_offset = layers.get("loc_x_offset").and_then(|f| f.as_f64()).unwrap_or(0.0);
        let loc_y_offset = layers.get("loc_y_offset").and_then(|f| f.as_f64()).unwrap_or(0.0);
        
        let neutral = get_json_string(layers, "neutral").to_string();
        let neutral_sc = get_json_string(layers, "neutral_sc").to_string();
        let border_margin_hint = layers.get("border_margin_hint").unwrap().as_f64().unwrap();

        LayerInfo {
            land_layer,
            island_layer,
            island_fill_layer,
            sea_layer,
            names_layer,
            centers_layer,
            units_layer,
            phantom_primary_armies_layer,
            phantom_retreat_armies_layer,
            phantom_primary_fleets_layer,
            phantom_retreat_fleets_layer,
            unit_type_labeled,
            province_labels,
            center_labels,
            unit_labels,
            neutral,
            neutral_sc,
            border_margin_hint,
            loc_x_offset,
            loc_y_offset
        }
    
    
    }
}