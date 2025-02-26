// import re
// from abc import abstractmethod
// from xml.etree.ElementTree import Element
// import numpy as np

// # TODO: Refactor all of this into one Transform (Empty and Translation can be trivally converted into Matrix)

use core::panic;

use quick_xml::events::BytesStart;

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

    pub fn transform(&self, p: (f64, f64)) -> (f64, f64) {
        let x = self.x_dx * p.0 + self.x_dy * p.1 + self.x_c;
        let y = self.y_dx * p.0 + self.y_dy * p.1 + self.y_c;
        (x, y)
    }

    pub fn inverse_tranform(&self, p: (f64, f64)) -> (f64, f64) {
        let p = (p.0 - self.x_c, p.1 - self.y_c);
        let det = self.x_dx * self.y_dy - self.x_dy * self.y_dx; 
        let x = self.y_dy * p.0 - self.x_dy * p.1;
        let y = - self.y_dx * p.0 + self.x_dx * p.1;
        (x / det, y / det)
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

    pub fn get_transform(e: BytesStart) -> Transform {
        let transform_string: Option<std::borrow::Cow<'_, str>> = get_attribute(&e, "tranform");

        if let Some(s) = transform_string {
            if s.len() == 0 {
                Self::empty()
            } else if s.starts_with("translate") {
                todo!()
                // if self.transform_string:
                // translation_match = re.search("^\\s*translate\\((.*),(.*)\\)\\s*", self.transform_string)
                // if translation_match:
                //     self.x_c = float(translation_match.group(1))
                //     self.y_c = float(translation_match.group(2))
                // else:
                //     raise RuntimeError("Translation not found")
            } else if s.starts_with("matrix") {
                todo!()
                // matrix_transform_match = re.search(
                //     "^\\s*matrix\\((.*),(.*),(.*),(.*),(.*),(.*)\\)\\s*", self.transform_string
                // )
                // self.matrix_transform: tuple[float, float, float, float, float, float] = (1, 0, 0, 1, 0, 0)
                // if matrix_transform_match:
                //     self.x_dx = float(matrix_transform_match.group(1))
                //     self.y_dx = float(matrix_transform_match.group(2))
                //     self.x_dy = float(matrix_transform_match.group(3))
                //     self.y_dy = float(matrix_transform_match.group(4))
                //     self.x_c = float(matrix_transform_match.group(5))
                //     self.y_c = float(matrix_transform_match.group(6))
                // else:
                //     raise RuntimeError("Matrix transform not found")
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
