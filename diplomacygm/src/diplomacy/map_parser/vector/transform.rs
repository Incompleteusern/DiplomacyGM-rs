// import re
// from abc import abstractmethod
// from xml.etree.ElementTree import Element
// import numpy as np

// # TODO: Refactor all of this into one Transform (Empty and Translation can be trivally converted into Matrix)

use core::panic;

use geo_types::Coord;
use quick_xml::events::BytesStart;
use regex::Regex;

use crate::diplomacy::map_parser::vector::utils::get_float;

use super::utils::get_attribute;

pub struct Transform {
    pub x_dx: f64,
    pub y_dy: f64,
    pub x_dy: f64,
    pub y_dx: f64,
    pub x_c: f64,
    pub y_c: f64
}

impl Transform {

    pub fn transform(&self, p: Coord) -> Coord {
        let x = self.x_dx * p.x + self.x_dy * p.y + self.x_c;
        let y = self.y_dx * p.x + self.y_dy * p.y + self.y_c;
        Coord { x: x, y: y }
    }

    pub fn inverse_transform(&self, p: Coord) -> Coord {
        let p = (p.x - self.x_c, p.y - self.y_c);
        let det = self.x_dx * self.y_dy - self.x_dy * self.y_dx; 
        let x = self.y_dy * p.0 - self.x_dy * p.1;
        let y = - self.y_dx * p.0 + self.x_dx * p.1;
        Coord { x: x / det, y: y / det }
    }

    pub fn empty() -> Transform {
        Transform { x_dx: 1.0, y_dy: 1.0, x_dy: 0.0, y_dx: 0.0, x_c: 0.0, y_c: 0.0 }
    }

    pub fn translate(x_c: f64, y_c: f64) -> Transform {
        Transform { x_dx: 1.0, y_dy: 1.0, x_dy: 0.0, y_dx: 0.0, x_c, y_c }
    }

    pub fn matrix(x_dx: f64, y_dy: f64, x_dy: f64, y_dx: f64, x_c: f64, y_c: f64) -> Transform {
        Transform { x_dx, y_dy, x_dy, y_dx, x_c, y_c }
    }

    pub fn get_transform(e: &BytesStart) -> Transform {
        let transform_string: Option<std::borrow::Cow<'_, str>> = get_attribute(e, "transform");
        // println!("transform: {:?}", transform_string);

        if let Some(s) = transform_string {
            if s.len() == 0 {
                Self::empty()
            } else if s.starts_with("translate") {
                let re = Regex::new(r"^\s*translate\(([^,]*),([^,]*)\)\s*$").unwrap();
                let m = re.captures(&s).expect("Translation not found");

                Self::translate(get_float(&m, 1), get_float(&m, 2))
            } else if s.starts_with("matrix") {
                let re = Regex::new(r"^\s*matrix\(([^,]*),([^,]*),([^,]*),([^,]*),([^,]*),([^,]*)\)\s*$").unwrap();
                let m = re.captures(&s).expect("Matrix transform not found");

                Self::matrix(get_float(&m, 1), get_float(&m, 4), get_float(&m, 3), get_float(&m, 2), get_float(&m, 5), get_float(&m, 6))
            } else {
                panic!("Unknown tranform: {}", s)
            }
        } else {
            Self::empty()
        }
    }
}


// def get_transform(element: Element) -> Transform:
//     transform_string: str | None = element.get("transform", None)
//     if not transform_string:
//         return EmptyTransform(element)
//     elif transform_string.startswith("translate"):
//         return Translation(element)
//     elif transform_string.startswith("matrix"):
//         return MatrixTransform(element)
//     else:
//         raise RuntimeError(f"Unknown transform: {transform_string}")
