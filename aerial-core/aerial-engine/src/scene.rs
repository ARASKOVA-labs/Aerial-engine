/// aerial-engine/src/scene.rs
/// Scene graph: stores all elements on the canvas.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Every type of element the Aerial engine can render.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ElementKind {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    FreeDraw,
    Text,
    Image,
    Diagram,
}

/// A single scene element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneElement {
    pub id: u64,
    pub kind: ElementKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Stroke color as a CSS string e.g. "#1a1a2e"
    pub stroke: String,
    /// Fill color as a CSS string or "transparent"
    pub fill: String,
    pub stroke_width: f64,
    /// For FreeDraw: a list of (x, y) points relative to (x, y)
    pub points: Vec<[f64; 2]>,
    /// For Image or PDF: The unique UUID pointing to the raw file in Tauri
    pub asset_id: Option<String>,
    /// For Text: The string content
    pub text: Option<String>,
    #[serde(default)]
    pub font_family: Option<String>,
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Whether this element acts as a highlighter (multiply blend mode)
    pub is_highlighter: bool,
    pub is_fountain_pen: bool,
    /// Sharpness value for the fountain pen (0.0 to 1.0)
    pub fountain_sharpness: f64,
    /// Whether this element acts as a laser pen (glowing stroke, usually deleted later)
    #[serde(default)]
    pub is_laser: bool,
    #[serde(default)]
    pub is_rough: bool,
    #[serde(default)]
    pub is_curved: bool,
    #[serde(default)]
    pub start_binding: Option<u64>,
    #[serde(default)]
    pub end_binding: Option<u64>,
    #[serde(default)]
    pub hit_map: Option<std::collections::HashMap<String, (f64, f64, f64, f64)>>,
}

impl SceneElement {
    pub fn new_rect(id: u64, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            id,
            kind: ElementKind::Rectangle,
            x,
            y,
            width: w,
            height: h,
            stroke: "#1a1a2e".to_string(),
            fill: "transparent".to_string(),
            stroke_width: 2.0,
            points: vec![],
            asset_id: None,
            text: None,
            font_family: None,
            alpha: None,
            is_highlighter: false,
            is_fountain_pen: false,
            fountain_sharpness: 0.0,
            is_laser: false,
            is_rough: false,
            is_curved: false,
            start_binding: None,
            end_binding: None,
            hit_map: None,
        }
    }

    pub fn new_free_draw(id: u64, x: f64, y: f64, points: Vec<[f64; 2]>) -> Self {
        Self {
            id,
            kind: ElementKind::FreeDraw,
            x,
            y,
            width: 0.0,
            height: 0.0,
            stroke: "#1a1a2e".to_string(),
            fill: "transparent".to_string(),
            stroke_width: 2.5,
            points,
            asset_id: None,
            text: None,
            font_family: None,
            alpha: None,
            is_highlighter: false,
            is_fountain_pen: false,
            fountain_sharpness: 0.0,
            is_laser: false,
            is_rough: false,
            is_curved: false,
            start_binding: None,
            end_binding: None,
            hit_map: None,
        }
    }

    pub fn new_text(id: u64, x: f64, y: f64, text: String, size: f64, font_family: Option<String>) -> Self {
        Self {
            id,
            kind: ElementKind::Text,
            x,
            y,
            width: size * 5.0, // rough estimate
            height: size,
            stroke: "#1a1a2e".to_string(),
            fill: "transparent".to_string(),
            stroke_width: 1.0,
            points: vec![],
            asset_id: None,
            text: Some(text),
            font_family,
            alpha: None,
            is_highlighter: false,
            is_fountain_pen: false,
            fountain_sharpness: 0.0,
            is_laser: false,
            is_rough: false,
            is_curved: false,
            start_binding: None,
            end_binding: None,
            hit_map: None,
        }
    }
}

/// Owns all elements for the current board.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub elements: Vec<SceneElement>,
    pub next_id: u64,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, el: SceneElement) {
        self.elements.push(el);
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Reverse iterate to find the top-most element that contains the point (x, y)
    pub fn hit_test(&self, px: f64, py: f64) -> Option<u64> {
        for el in self.elements.iter().rev() {
            if el.contains(px, py) {
                return Some(el.id);
            }
        }
        None
    }

    pub fn get_element_mut(&mut self, id: u64) -> Option<&mut SceneElement> {
        self.elements.iter_mut().find(|e| e.id == id)
    }

    pub fn get_element(&self, id: u64) -> Option<&SceneElement> {
        self.elements.iter().find(|e| e.id == id)
    }
}

impl SceneElement {
    /// Bounding box / shape hit testing
    pub fn contains(&self, px: f64, py: f64) -> bool {
        let threshold = (self.stroke_width / 2.0).max(4.0);
        match self.kind {
            ElementKind::Rectangle | ElementKind::Image | ElementKind::Diagram => {
                px >= self.x - threshold && px <= self.x + self.width + threshold &&
                py >= self.y - threshold && py <= self.y + self.height + threshold
            },
            ElementKind::Ellipse => {
                let cx = self.x + self.width / 2.0;
                let cy = self.y + self.height / 2.0;
                let rx = (self.width.abs() / 2.0) + threshold;
                let ry = (self.height.abs() / 2.0) + threshold;
                if rx <= 0.0 || ry <= 0.0 { return false; }
                let dx = px - cx;
                let dy = py - cy;
                (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
            },
            ElementKind::Line | ElementKind::Arrow => {
                // simple bounding box for line for now
                let min_x = self.x.min(self.x + self.width) - threshold;
                let max_x = self.x.max(self.x + self.width) + threshold;
                let min_y = self.y.min(self.y + self.height) - threshold;
                let max_y = self.y.max(self.y + self.height) + threshold;
                px >= min_x && px <= max_x && py >= min_y && py <= max_y
            },
            ElementKind::FreeDraw => {
                if self.points.is_empty() { return false; }
                let mut min_x = self.x;
                let mut max_x = self.x;
                let mut min_y = self.y;
                let mut max_y = self.y;
                for pt in &self.points {
                    let absolute_x = self.x + pt[0];
                    let absolute_y = self.y + pt[1];
                    if absolute_x < min_x { min_x = absolute_x; }
                    if absolute_x > max_x { max_x = absolute_x; }
                    if absolute_y < min_y { min_y = absolute_y; }
                    if absolute_y > max_y { max_y = absolute_y; }
                }
                px >= min_x - threshold && px <= max_x + threshold &&
                py >= min_y - threshold && py <= max_y + threshold
            },
            ElementKind::Text => {
                let w = self.width.abs().max(60.0);
                let h = self.height.abs().max(30.0);
                px >= self.x - threshold && px <= self.x + w + threshold &&
                py >= self.y - threshold && py <= self.y + h + threshold
            },
        }
    }
    
    /// Like contains() but with a custom radius for the eraser tool.
    pub fn contains_radius(&self, px: f64, py: f64, radius: f64) -> bool {
        match self.kind {
            ElementKind::Rectangle | ElementKind::Image | ElementKind::Diagram => {
                let min_x = self.x.min(self.x + self.width) - radius;
                let max_x = self.x.max(self.x + self.width) + radius;
                let min_y = self.y.min(self.y + self.height) - radius;
                let max_y = self.y.max(self.y + self.height) + radius;
                px >= min_x && px <= max_x && py >= min_y && py <= max_y
            },
            ElementKind::Ellipse => {
                let cx = self.x + self.width / 2.0;
                let cy = self.y + self.height / 2.0;
                let rx = self.width.abs() / 2.0 + radius;
                let ry = self.height.abs() / 2.0 + radius;
                if rx <= 0.0 || ry <= 0.0 { return false; }
                let dx = px - cx;
                let dy = py - cy;
                (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
            },
            ElementKind::Line | ElementKind::Arrow => {
                let min_x = self.x.min(self.x + self.width) - radius;
                let max_x = self.x.max(self.x + self.width) + radius;
                let min_y = self.y.min(self.y + self.height) - radius;
                let max_y = self.y.max(self.y + self.height) + radius;
                px >= min_x && px <= max_x && py >= min_y && py <= max_y
            },
            ElementKind::FreeDraw => {
                if self.points.is_empty() { return false; }
                let mut min_x = self.x;
                let mut max_x = self.x;
                let mut min_y = self.y;
                let mut max_y = self.y;
                for pt in &self.points {
                    let ax = self.x + pt[0];
                    let ay = self.y + pt[1];
                    if ax < min_x { min_x = ax; }
                    if ax > max_x { max_x = ax; }
                    if ay < min_y { min_y = ay; }
                    if ay > max_y { max_y = ay; }
                }
                px >= min_x - radius && px <= max_x + radius &&
                py >= min_y - radius && py <= max_y + radius
            },
            ElementKind::Text => {
                let w = self.width.abs().max(60.0);
                let h = self.height.abs().max(30.0);
                px >= self.x - radius && px <= self.x + w + radius &&
                py >= self.y - radius && py <= self.y + h + radius
            },
        }
    }

    /// Checks if this element intersects with or is fully contained by a rectangle defined by (rx, ry, rw, rh)
    pub fn intersects_rect(&self, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
        let min_rx = rx.min(rx + rw);
        let max_rx = rx.max(rx + rw);
        let min_ry = ry.min(ry + rh);
        let max_ry = ry.max(ry + rh);

        match self.kind {
            ElementKind::Rectangle | ElementKind::Ellipse | ElementKind::Image | ElementKind::Diagram | ElementKind::Line | ElementKind::Arrow | ElementKind::Text => {
                let w = if self.kind == ElementKind::Text { self.width.abs().max(60.0) } else { self.width };
                let h = if self.kind == ElementKind::Text { self.height.abs().max(30.0) } else { self.height };
                let min_x = self.x.min(self.x + w);
                let max_x = self.x.max(self.x + w);
                let min_y = self.y.min(self.y + h);
                let max_y = self.y.max(self.y + h);
                
                !(max_rx < min_x || min_rx > max_x || max_ry < min_y || min_ry > max_y)
            },
            ElementKind::FreeDraw => {
                if self.points.is_empty() { return false; }
                let mut min_x = self.x;
                let mut max_x = self.x;
                let mut min_y = self.y;
                let mut max_y = self.y;
                for pt in &self.points {
                    let px = self.x + pt[0];
                    let py = self.y + pt[1];
                    if px < min_x { min_x = px; }
                    if px > max_x { max_x = px; }
                    if py < min_y { min_y = py; }
                    if py > max_y { max_y = py; }
                }
                !(max_rx < min_x || min_rx > max_x || max_ry < min_y || min_ry > max_y)
            }
        }
    }
}
