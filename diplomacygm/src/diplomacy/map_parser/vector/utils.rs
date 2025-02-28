

// pub const NAMESPACE_INKSCAPE: &str = "{http://www.inkscape.org/namespaces/inkscape}";
// pub const NAMESPACE_SODIPODI: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd";
// pub const NAMESPACE_SVG: &str = "http://www.w3.org/2000/svg";

pub const INKSPACE_LABEL: &str = "inkscape:label";
pub const SODIPODI_SIDES: &str = "sodipodi:sides";
pub const SODIPODI_CX: &str = "sodipodi:cx";
pub const SODIPODI_CY: &str = "sodipodi:cy";

// logger = logging.getLogger(__name__)

use std::{borrow::Cow, collections::HashMap, sync::Arc};

use geo_types::{Coord, LineString, Polygon};
use quick_xml::events::BytesStart;
use regex::Captures;
use serde_json::Value;

use crate::diplomacy::persistence::player::PlayerInfo;

use super::transform::Transform;

pub fn get_float(m: &Captures, i: usize) -> f64 {
    m.get(i).unwrap().as_str().parse::<f64>().expect("Failed to parse float")
}

pub fn get_attribute<'a>(e: &'a BytesStart, id: &str) -> Option<Cow<'a, str>> {
    e.try_get_attribute(id).unwrap_or_else(|_| panic!("Failed to get attribute \'{}\' in svg", id))
        .map(|f| f.unescape_value().expect("Failed to unescape value in svg"))    
}

pub fn get_id<'a>(e: &'a BytesStart) -> Cow<'a, str> {
    get_attribute(e, "id").unwrap()
}

pub fn get_inkspace_label<'a>(e: &'a BytesStart) -> Cow<'a, str> {
    // println!("{:?}", e);
    // for a in e.attributes() {
    //     println!("({:?})", a.unwrap().key)
    // }
    get_attribute(e, INKSPACE_LABEL).unwrap()
}

pub fn get_fill_color(style: &str) -> Option<&str> {
    for s in style.split(";") {
        let potential_color = s.strip_prefix("fill:#");
        if potential_color.is_some() {
            return potential_color;
        }
    }

    None
}

pub fn get_player(e: &BytesStart, color_to_player: &HashMap<String, Option<Arc<PlayerInfo>>>) -> Option<Arc<PlayerInfo>> {
    let style_attribute = get_attribute(e, "style")?;
    let fill_color = get_fill_color(&style_attribute)?;
    color_to_player.get(fill_color)?.clone()
}



// def get_player(element: Element, color_to_player: dict[str, Player]) -> Player:
//     return color_to_player[get_element_color(element)]



pub fn get_json_string<'a>(data: &'a Value, string: &'a str) -> &'a str {
     data.get(string).unwrap_or_else(|| panic!("Expected \'{}\' in json", string)).as_str().unwrap_or_else(|| panic!("Expected \'{}\' as string", string))
}

// def get_element_color(element: Element) -> str:
//     style = element.get("style").split(";")
//     for value in style:
//         prefix = "fill:#"
//         if value.startswith(prefix):
//             return value[len(prefix) :]


// def get_player(element: Element, color_to_player: dict[str, Player]) -> Player:
//     return color_to_player[get_element_color(element)]

pub fn get_unit_coordinates(e: &BytesStart<'_>) -> Coord {
    let x = get_attribute(e, SODIPODI_CX).map(|f| f.into_owned().parse::<f64>().unwrap());
    let y = get_attribute(e, SODIPODI_CY).map(|f| f.into_owned().parse::<f64>().unwrap());

    if x.is_none() || y.is_none() {
        todo!();
    }

    Transform::get_transform(e).transform(Coord { x: x.unwrap(), y: y.unwrap() })
}

// def get_unit_coordinates(
//     unit_data: Element,
// ) -> tuple[float, float]:
//     path: Element = unit_data.find("{http://www.w3.org/2000/svg}path")

//     x = path.get("{http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd}cx")
//     y = path.get("{http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd}cy")
//     if x == None or y == None:
//         # find all the points the objects are at
//         # take the center of the bounding box
//         for path in unit_data.findall("{http://www.w3.org/2000/svg}path"):
//             pathstr = path.get("d")
//             coordinates = parse_path(pathstr, EmptyTransform(None), get_transform(path))
//             coordinates = np.array(sum(coordinates, start = []))
//             minp = np.min(coordinates, axis=0)
//             maxp = np.max(coordinates, axis=0)
//             return ((minp + maxp) / 2).tolist()

//     else:
//         x = float(x)
//         y = float(y)
//         return get_transform(path).transform((x, y))


// def move_coordinate(
//     former_coordinate: tuple[float, float],
//     coordinate: tuple[float, float],
// ) -> tuple[float, float]:
//     return (former_coordinate[0] + coordinate[0], former_coordinate[1] + coordinate[1])

fn parse_path_command(
    command: char,
    offset: (f64, f64),
    current_coord: Coord,
) -> Coord {
    let reset = command.is_ascii_uppercase();
    let command = command.to_ascii_lowercase();

    match command {
        'm' | 'c' | 'l' | 't' | 's' | 'q' | 'a' => {
            if reset {
                Coord { x: offset.0, y: offset.1 }
            } else {
                Coord { x: current_coord.x + offset.0, y: current_coord.y + offset.1 }
            }
        },
        'v' => {
            if reset {
                Coord { x: current_coord.x, y: offset.1 }
            } else {
                Coord { x: current_coord.x, y: current_coord.y + offset.1 }
            }
        },
        'h' => {
            if reset {
                Coord { x: offset.0, y: current_coord.y }
            } else {
                Coord { x: current_coord.x + offset.0, y: current_coord.y }
            }
        },
        _ => panic!("Unknown SVG path command: {}", command)
    }

    //     if command in ["m", "c", "l", "t", "s", "q", "a"]:
//         if reset:
//             coordinate = (0, 0)
//         return move_coordinate(coordinate, args[-1])  # Ignore all args except the last
//     elif command in ["h", "v"]:
//         coordinate = list(coordinate)
//         if command == "h":
//             index = 0
//         else:
//             index = 1
//         if reset:
//             coordinate[index] = 0
//         coordinate[index] += args[0][0]
//         return tuple(coordinate)

}

// # returns:
// # new base_coordinate (= base_coordinate if not applicable),
// # new former_coordinate (= former_coordinate if not applicable),
// def _parse_path_command(
//     command: str,
//     args: list[tuple[float, float]],
//     coordinate: tuple[float, float],
// ) -> tuple[tuple[float, float], tuple[float, float]]:
//     reset = command.isupper()
//     command = command.lower()

//     if command in ["m", "c", "l", "t", "s", "q", "a"]:
//         if reset:
//             coordinate = (0, 0)
//         return move_coordinate(coordinate, args[-1])  # Ignore all args except the last
//     elif command in ["h", "v"]:
//         coordinate = list(coordinate)
//         if command == "h":
//             index = 0
//         else:
//             index = 1
//         if reset:
//             coordinate[index] = 0
//         coordinate[index] += args[0][0]
//         return tuple(coordinate)
//     else:
//         raise RuntimeError(f"Unknown SVG path command: {command}")

pub fn parse_path(path_string: &str, layer_transform: &Transform, this_transform: &Transform) -> Vec<Polygon> {
    // println!("{}", path_string);

    let mut path: Vec<String> = Vec::new();

    for s in path_string.split(" ") {
        if s.chars().next().unwrap().is_ascii_alphabetic() && s.len() > 1 {
            let (command, argument) = s.split_at(1);

            path.push(command.to_string());
            path.push(argument.to_string());
        } else {
            path.push(s.to_string());
        }

    }

    let mut province_polygons = Vec::new();
    let mut current_polygon = Vec::new();
    let mut command: Option<char> = None;
    let mut expected_arguments = 0;
    let mut start: Option<Coord> = None;
    let mut coordinate = Coord { x: 0.0, y: 0.0};

    let mut path_iter = path.iter().peekable();

    while let Some(s) = path_iter.peek() {
        let first_char = s.chars().next().unwrap();

        if first_char.is_ascii_alphabetic() {
            path_iter.next();
            command = Some(first_char);

            match first_char.to_ascii_lowercase() {
                'z' => {
                    if let Some(point) = start {
                        current_polygon.push(layer_transform.transform(this_transform.transform(point)));
                    } else {
                        panic!("Invalid geometry: got 'z' on first element in a subgeometry");
                    }

                    start = None;
                    province_polygons.push(Polygon::new(LineString::new(current_polygon), Vec::new()));
                    current_polygon = Vec::new();

                    // If we are closing, and there is more, there must be a second polygon (Chukchi Sea)
                    if path_iter.peek().is_some() {
                        continue;
                    } else {
                        break;
                    }
                },
                'm' | 'l' | 'h' | 'v' | 't' => expected_arguments = 1,
                's' | 'q' => expected_arguments = 2,
                'c' => expected_arguments = 3,
                'a' => expected_arguments = 4,
                _ => panic!("Unknown SVG path command {}", first_char)
            }
        }

        if command == Some('z') {
            panic!("Invalid path, 'z' was followed by arguments")
        }

        for _ in 0..(expected_arguments-1) {
            let _ = path_iter.next().unwrap();
            // println!("skipping {}", coord);
        }
        
        let offset = path_iter.next().unwrap();

        let offset = {
            match command.unwrap().to_ascii_lowercase() {
                'v' => {
                    (0.0, offset.parse::<f64>().unwrap())
                },
                'h' => {
                    (offset.parse::<f64>().unwrap(), 0.0)
                },
                _ => {
                    let (x, y) = offset.split_once(",").unwrap();
                    (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
                }
            }
        };
        // println!("{:?}", offset);

        coordinate = parse_path_command(command.unwrap(), offset, coordinate);

        start = start.or(Some(coordinate));
        
        current_polygon.push(layer_transform.transform(this_transform.transform(coordinate)));
    }

    if !current_polygon.is_empty() {
        province_polygons.push(Polygon::new(LineString::new(current_polygon), Vec::new()));
    }


    province_polygons

}

// def parse_path(path_string: str, layer_translation: Transform, this_translation: Transform):
//     province_coordinates = [[]]
//     command = None
//     expected_arguments = 0
//     current_index = 0
//     path: list[str] = path_string.split()

//     start = None
//     coordinate = (0, 0)
//     while current_index < len(path):
//         if path[current_index][0].isalpha():
//             if len(path[current_index]) != 1:
//                 # m20,70 is valid syntax, so move the 20,70 to the next element
//                 path.insert(current_index + 1, path[current_index][1:])
//                 path[current_index] = path[current_index][0]

//             command = path[current_index]
//             if command.lower() == "z":
//                 if start == None:
//                     raise Exception("Invalid geometry: got 'z' on first element in a subgeometry")
//                 province_coordinates[-1].append(start)
//                 start = None
//                 current_index += 1
//                 if current_index < len(path):
//                     # If we are closing, and there is more, there must be a second polygon (Chukchi Sea)
//                     province_coordinates += [[]]
//                     continue
//                 else:
//                     break

//             elif command.lower() in ["m", "l", "h", "v", "t"]:
//                 expected_arguments = 1
//             elif command.lower() in ["s", "q"]:
//                 expected_arguments = 2
//             elif command.lower() in ["c"]:
//                 expected_arguments = 3
//             elif command.lower() in ["a"]:
//                 expected_arguments = 4
//             else:
//                 raise RuntimeError(f"Unknown SVG path command {command}")

//             current_index += 1

//         if command.lower() == "z":
//             raise Exception("Invalid path, 'z' was followed by arguments")

//         if len(path) < (current_index + expected_arguments):
//             raise RuntimeError(f"Ran out of arguments for {command}")

//         args = [
//             (float(coord_string.split(",")[0]), float(coord_string.split(",")[-1]))
//             for coord_string in path[current_index : current_index + expected_arguments]
//         ]

//         coordinate = _parse_path_command(
//             command, args, coordinate
//         )

//         if start == None:
//             start = coordinate

//         province_coordinates[-1].append(layer_translation.transform(this_translation.transform(coordinate)))
//         current_index += expected_arguments
//     return province_coordinates