/*!
 * aerial-engine/src/canvas.rs
 * Proprietary 2D Canvas Rendering Pipeline
 * © ARASKOVA Labs — All rights reserved.
 *
 * AerialCanvas is the central controller that owns the scene, manages
 * the active tool, and drives all draw calls directly to an HTML5 Canvas
 * via the WebAssembly–JavaScript bridge.
 */

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent, HtmlImageElement};
use yrs::{Doc, Map, MapRef, Transact, StateVector, Update, ReadTxn};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;

use crate::scene::{Scene, SceneElement, ElementKind};
use crate::tools::ActiveTool;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Background color for the canvas — Araskova bone-white
const BG_COLOR: &str = "#FAFAF8";
/// Default stroke color
const STROKE_COLOR: &str = "#1a1a2e";
/// Selection highlight color
const SELECTION_COLOR: &str = "#6366f1";
/// Grid dot color
const GRID_DOT_COLOR: &str = "rgba(0,0,0,0.05)";
/// Grid spacing in logical pixels
const GRID_SPACING: f64 = 24.0;

// ─── AerialCanvas ─────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct AerialCanvas {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    scene: Scene,
    active_tool: ActiveTool,
    /// Is the user currently drawing/dragging?
    is_drawing: bool,
    /// Draw start X (canvas coordinates)
    start_x: f64,
    /// Draw start Y (canvas coordinates)
    start_y: f64,
    /// In-progress free-draw points
    current_points: Vec<[f64; 2]>,
    /// Pan offset X
    pan_x: f64,
    /// Pan offset Y
    pan_y: f64,
    /// Zoom scale factor
    zoom: f64,
    /// Currently selected element IDs
    selected_ids: Vec<u64>,
    /// Selection box start X, Y
    selection_box_start: Option<(f64, f64)>,
    /// Selection box current X, Y
    selection_box_current: Option<(f64, f64)>,
    /// Pan/Move start state
    drag_start_screen_x: f64,
    drag_start_screen_y: f64,
    drag_start_pan_x: f64,
    drag_start_pan_y: f64,
    drag_start_el_x: f64,
    drag_start_el_y: f64,
    drag_start_el_w: f64,
    drag_start_el_h: f64,
    drag_start_elements: Vec<SceneElement>,
    /// Are we currently resizing?
    is_resizing: bool,
    current_x: f64,
    current_y: f64,
    /// Current stroke color (CSS string)
    stroke_color: String,
    /// Current fill color (CSS string)
    fill_color: String,
    /// Current stroke width
    stroke_width: f64,
    
    #[wasm_bindgen(skip)]
    pub image_cache: HashMap<u64, HtmlImageElement>,
    
    #[wasm_bindgen(skip)]
    pub raster_cache: std::cell::RefCell<HashMap<u64, HtmlCanvasElement>>,
    
    #[wasm_bindgen(skip)]
    pub doc: Doc,
    #[wasm_bindgen(skip)]
    pub elements_map: MapRef,
    
    /// Flag indicating if the board state has mutated since last save
    is_dirty: bool,
    
    /// Is dark mode active?
    dark_mode: bool,
    /// Type of grid: "dots", "lines", "crosses", "blank"
    grid_type: String,
    /// Calligraphy fountain pen offset sharpness
    fountain_sharpness: f64,
    /// Store IDs of strokes drawn with the Magic Pen for extraction
    magic_stroke_ids: Vec<u64>,
    is_rough: bool,
    is_curved: bool,
    eraser_size: f64,
    current_start_binding: Option<u64>,
    current_end_binding: Option<u64>,
}

#[wasm_bindgen]
impl AerialCanvas {
    /// Create a new AerialCanvas that renders onto the canvas element with the given id.
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<AerialCanvas, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let el = document
            .get_element_by_id(canvas_id)
            .ok_or("canvas element not found")?;
        let canvas: HtmlCanvasElement = el.dyn_into()?;
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or("could not get 2d context")?
            .dyn_into()?;

        let doc = Doc::new();
        let elements_map = doc.get_or_insert_map("elements");

        Ok(AerialCanvas {
            canvas,
            ctx,
            scene: Scene::new(),
            active_tool: ActiveTool::FreeDraw,
            is_drawing: false,
            start_x: 0.0,
            start_y: 0.0,
            current_points: vec![],
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            selected_ids: Vec::new(),
            selection_box_start: None,
            selection_box_current: None,
            drag_start_screen_x: 0.0,
            drag_start_screen_y: 0.0,
            drag_start_pan_x: 0.0,
            drag_start_pan_y: 0.0,
            drag_start_el_x: 0.0,
            drag_start_el_y: 0.0,
            drag_start_el_w: 0.0,
            drag_start_el_h: 0.0,
            drag_start_elements: Vec::new(),
            is_resizing: false,
            current_x: 0.0,
            current_y: 0.0,
            stroke_color: STROKE_COLOR.to_string(),
            fill_color: "transparent".to_string(),
            stroke_width: 2.5,
            image_cache: HashMap::new(),
            raster_cache: std::cell::RefCell::new(HashMap::new()),
            doc,
            elements_map,
            is_dirty: false,
            dark_mode: false,
            grid_type: "dots".to_string(),
            fountain_sharpness: 0.5,
            magic_stroke_ids: Vec::new(),
            is_rough: false,
            is_curved: false,
            eraser_size: 40.0,
            current_start_binding: None,
            current_end_binding: None,
        })
    }

    // ── Tool selection ──────────────────────────────────────────────────────
    
    pub fn delete_selected(&mut self) {
        if !self.selected_ids.is_empty() {
            let ids = self.selected_ids.clone();
            for id in ids {
                self.delete_element(id);
            }
        }
    }

    pub fn delete_element(&mut self, id: u64) {
        let mut txn = self.doc.transact_mut();
        self.elements_map.remove(&mut txn, id.to_string().as_str());
        self.scene.elements.retain(|el| el.id != id);
        self.selected_ids.retain(|&sel_id| sel_id != id);
        self.is_dirty = true;
        self.render();
    }

    pub fn on_wheel(&mut self, dx: f64, dy: f64, ctrl: bool, screen_x: f64, screen_y: f64) {
        if ctrl {
            // Zoom (pinch-to-zoom emits ctrl=true on trackpads, or ctrl+wheel on mouse)
            let zoom_speed = 0.005;
            let zoom_delta = -dy * zoom_speed;
            let old_zoom = self.zoom;
            self.zoom = (self.zoom + zoom_delta).max(0.1).min(10.0);
            
            // Adjust pan so we zoom in on the cursor
            self.pan_x = screen_x - (screen_x - self.pan_x) * (self.zoom / old_zoom);
            self.pan_y = screen_y - (screen_y - self.pan_y) * (self.zoom / old_zoom);
        } else {
            // Pan (two-finger swipe)
            self.pan_x -= dx;
            self.pan_y -= dy;
        }
        self.render();
    }

    pub fn set_tool_freedraw(&mut self)   { self.active_tool = ActiveTool::FreeDraw; }
    pub fn set_tool_rectangle(&mut self)  { self.active_tool = ActiveTool::Rectangle; }
    pub fn set_tool_ellipse(&mut self)    { self.active_tool = ActiveTool::Ellipse; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_line(&mut self)       { self.active_tool = ActiveTool::Line; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_arrow(&mut self)      { self.active_tool = ActiveTool::Arrow; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_select(&mut self)     { self.active_tool = ActiveTool::Select; }
    pub fn set_tool_hand(&mut self)       { self.active_tool = ActiveTool::Hand; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_highlighter(&mut self) { self.active_tool = ActiveTool::Highlighter; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_fountain_pen(&mut self) { self.active_tool = ActiveTool::FountainPen; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_text(&mut self) { self.active_tool = ActiveTool::Text; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_eraser(&mut self) { self.active_tool = ActiveTool::Eraser; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_magic_pen(&mut self) { self.active_tool = ActiveTool::MagicPen; self.selected_ids.clear(); self.render(); }
    pub fn set_tool_laser_pen(&mut self) { self.active_tool = ActiveTool::LaserPen; self.selected_ids.clear(); self.render(); }

    /// Extracts strokes drawn with the Magic Pen in Google Input Tools format:
    /// [[[x1, x2, ...], [y1, y2, ...]], ...] and deletes them from the canvas.
    pub fn extract_magic_strokes(&mut self) -> String {
        let mut strokes_json = Vec::new();
        let mut txn = self.doc.transact_mut();
        
        for id in &self.magic_stroke_ids {
            if let Some(el) = self.scene.elements.iter().find(|e| e.id == *id) {
                let mut xs = Vec::new();
                let mut ys = Vec::new();
                
                // Add the starting point
                xs.push(el.x);
                ys.push(el.y);
                
                // Add all relative points
                for p in &el.points {
                    xs.push(el.x + p[0]);
                    ys.push(el.y + p[1]);
                }
                
                strokes_json.push(serde_json::json!([xs, ys]));
                
                // Remove from Yrs CRDT map
                self.elements_map.remove(&mut txn, id.to_string().as_str());
            }
        }
        
        // Remove from local scene
        let ids = self.magic_stroke_ids.clone();
        self.scene.elements.retain(|el| !ids.contains(&el.id));
        self.magic_stroke_ids.clear();
        
        if !ids.is_empty() {
            self.is_dirty = true;
            self.render();
        }
        
        serde_json::to_string(&strokes_json).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn clear_laser_strokes(&mut self) {
        let mut txn = self.doc.transact_mut();
        let laser_ids: Vec<u64> = self.scene.elements.iter()
            .filter(|el| el.is_laser)
            .map(|el| el.id)
            .collect();
            
        if !laser_ids.is_empty() {
            for id in &laser_ids {
                self.elements_map.remove(&mut txn, id.to_string().as_str());
            }
            self.scene.elements.retain(|el| !laser_ids.contains(&el.id));
            self.is_dirty = true;
            self.render();
        }
    }

    pub fn tick_animations(&mut self) -> bool {
        let mut needs_render = false;
        let mut ids_to_remove = Vec::new();

        for el in self.scene.elements.iter_mut() {
            if el.is_laser {
                if let Some(alpha) = el.alpha {
                    let new_alpha = alpha - 0.015; // Fade out slightly (~1s at 60fps)
                    if new_alpha <= 0.0 {
                        ids_to_remove.push(el.id);
                    } else {
                        el.alpha = Some(new_alpha);
                        needs_render = true;
                    }
                }
            }
        }

        if !ids_to_remove.is_empty() {
            let mut txn = self.doc.transact_mut();
            for id in &ids_to_remove {
                self.elements_map.remove(&mut txn, id.to_string().as_str());
            }
            self.scene.elements.retain(|el| !ids_to_remove.contains(&el.id));
            self.is_dirty = true;
            needs_render = true;
        }

        if needs_render {
            self.render();
        }
        
        needs_render
    }

    // ── Save & Load ────────────────────────────────────────────────────────
    
    pub fn check_and_clear_dirty(&mut self) -> bool {
        let dirty = self.is_dirty;
        self.is_dirty = false;
        dirty
    }

    pub fn get_scene_json(&self) -> String {
        serde_json::to_string(&self.scene).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn load_scene_json(&mut self, json: &str) {
        if let Ok(scene) = serde_json::from_str::<Scene>(json) {
            self.scene = scene;
            
            // Sync loaded scene to CRDT YMap
            let mut txn = self.doc.transact_mut();
            for el in &self.scene.elements {
                if let Ok(json_str) = serde_json::to_string(el) {
                    self.elements_map.insert(&mut txn, el.id.to_string(), json_str);
                }
            }
            self.is_dirty = true;
            self.render();
        }
    }

    pub fn export_full_state(&mut self) -> Vec<u8> {
        let mut txn = self.doc.transact_mut();
        for el in &self.scene.elements {
            if let Ok(json_str) = serde_json::to_string(el) {
                self.elements_map.insert(&mut txn, el.id.to_string(), json_str);
            }
        }
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    pub fn get_local_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = txn.state_vector();
        
        let mut packet = vec![0u8];
        packet.extend_from_slice(&sv.encode_v1());
        packet
    }

    pub fn process_incoming_packet(&mut self, packet: &[u8]) -> Option<Vec<u8>> {
        if packet.is_empty() { return None; }
        
        let message_type = packet[0];
        let data = &packet[1..];

        match message_type {
            0 => {
                let txn = self.doc.transact();
                let remote_sv = StateVector::decode_v1(data).unwrap_or_default();
                let delta = txn.encode_state_as_update_v1(&remote_sv);
                
                let mut reply = vec![1u8];
                reply.extend_from_slice(&delta);
                Some(reply)
            },
            1 => {
                if let Ok(update) = Update::decode_v1(data) {
                    let mut txn = self.doc.transact_mut();
                    let _ = txn.apply_update(update);
                    
                    let mut elements = Vec::new();
                    for (_key, value) in self.elements_map.iter(&txn) {
                        if let yrs::types::Value::Any(yrs::Any::String(s)) = value {
                            if let Ok(el) = serde_json::from_str::<SceneElement>(&s.to_string()) {
                                elements.push(el);
                            }
                        }
                    }
                    let max_id = elements.iter().map(|e| e.id).max().unwrap_or(0);
                    if max_id >= self.scene.next_id {
                        self.scene.next_id = max_id;
                    }
                    elements.sort_by(|a, b| a.id.cmp(&b.id));
                    
                    self.scene.elements = elements;
                    self.selected_ids.clear();
                    self.render();
                }
                None
            },
            _ => None
        }
    }

    pub fn apply_remote_delta(&mut self, delta_bytes: &[u8]) {
        if let Ok(update) = Update::decode_v1(delta_bytes) {
            let mut txn = self.doc.transact_mut();
            let _ = txn.apply_update(update);
            
            // Rebuild scene elements from YMap
            let mut elements = Vec::new();
            for (_key, value) in self.elements_map.iter(&txn) {
                if let yrs::types::Value::Any(yrs::Any::String(s)) = value {
                    if let Ok(el) = serde_json::from_str::<SceneElement>(&s.to_string()) {
                        elements.push(el);
                    }
                }
            }
            let max_id = elements.iter().map(|e| e.id).max().unwrap_or(0);
            if max_id >= self.scene.next_id {
                self.scene.next_id = max_id;
            }
            elements.sort_by(|a, b| a.id.cmp(&b.id));
            
            self.scene.elements = elements;
            self.selected_ids.clear();
            // ⚡ DO NOT flip is_dirty to true here! 
            // This prevents infinite local storage write loops from network inputs.
            self.render();
        }
    }

    pub fn import_full_state(&mut self, bytes: &[u8]) {
        if let Ok(update) = Update::decode_v1(bytes) {
            let mut txn = self.doc.transact_mut();
            let _ = txn.apply_update(update);
            
            let mut elements = Vec::new();
            for (_key, value) in self.elements_map.iter(&txn) {
                if let yrs::types::Value::Any(yrs::Any::String(s)) = value {
                    if let Ok(el) = serde_json::from_str::<SceneElement>(&s.to_string()) {
                        elements.push(el);
                    }
                }
            }
            let max_id = elements.iter().map(|e| e.id).max().unwrap_or(0);
            if max_id >= self.scene.next_id {
                self.scene.next_id = max_id;
            }
            elements.sort_by(|a, b| a.id.cmp(&b.id));
            
            self.scene.elements = elements;
            self.selected_ids.clear();
            self.is_dirty = false;
            self.render();
        }
    }

    // ── Style controls ─────────────────────────────────────────────────────

    pub fn set_stroke_color(&mut self, color: &str)  { self.stroke_color = color.to_string(); }
    pub fn set_fill_color(&mut self, color: &str)    { self.fill_color = color.to_string(); }
    pub fn set_stroke_width(&mut self, w: f64) {
        self.stroke_width = w;
    }

    pub fn set_fountain_sharpness(&mut self, s: f64) {
        self.fountain_sharpness = s;
    }

    pub fn set_is_rough(&mut self, rough: bool) {
        self.is_rough = rough;
    }

    pub fn set_is_curved(&mut self, curved: bool) {
        self.is_curved = curved;
    }

    pub fn set_eraser_size(&mut self, size: f64) {
        self.eraser_size = size;
    }

    pub fn set_dark_mode(&mut self, dark: bool) {
        self.dark_mode = dark;
        self.render();
    }
    
    pub fn set_grid_type(&mut self, gtype: &str) {
        self.grid_type = gtype.to_string();
        self.render();
    }

    // ── Viewport controls ──────────────────────────────────────────────────

    pub fn zoom_in(&mut self)  { self.zoom = (self.zoom * 1.1).min(8.0); self.render(); }
    pub fn zoom_out(&mut self) { self.zoom = (self.zoom / 1.1).max(0.1); self.render(); }
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.render();
    }
    
    pub fn screen_to_world_x(&self, screen_x: f64) -> f64 {
        (screen_x - self.pan_x) / self.zoom
    }
    
    pub fn screen_to_world_y(&self, screen_y: f64) -> f64 {
        (screen_y - self.pan_y) / self.zoom
    }

    pub fn clear_board(&mut self) {
        self.scene.clear();
        let mut txn = self.doc.transact_mut();
        let keys: Vec<String> = self.elements_map.keys(&txn).map(|k| k.to_string()).collect();
        for key in keys {
            self.elements_map.remove(&mut txn, key.as_str());
        }
        self.is_dirty = true;
        self.render();
    }

    pub fn add_text(&mut self, text: &str, x: f64, y: f64, size: f64, font_family: Option<String>) {
        let mut el = SceneElement::new_text(self.scene.next_id(), x, y, text.to_string(), size, font_family.clone());
        
        let default_font = "'Space Grotesk', sans-serif".to_string();
        let font_name = font_family.as_ref().unwrap_or(&default_font);
        self.ctx.set_font(&format!("{}px {}", size, font_name));
        
        let lines: Vec<&str> = text.split('\n').collect();
        let mut max_width = 0.0;
        for line in &lines {
            // Rough heuristic since web-sys measure_text feature is missing
            let w = line.chars().count() as f64 * (size * 0.55);
            if w > max_width { max_width = w; }
        }
        el.width = max_width;
        el.height = size * 1.2 * lines.len() as f64;
        
        self.scene.add(el);
        self.is_dirty = true;
        self.render();
    }

    // ── Mouse event handlers ───────────────────────────────────────────────

    pub fn on_mouse_down(&mut self, event: &MouseEvent) {
        let (x, y) = self.event_to_canvas(event);
        self.is_drawing = true;
        self.start_x = x;
        self.start_y = y;
        
        let rect = self.canvas.get_bounding_client_rect();
        self.drag_start_screen_x = event.client_x() as f64 - rect.left();
        self.drag_start_screen_y = event.client_y() as f64 - rect.top();
        self.drag_start_pan_x = self.pan_x;
        self.drag_start_pan_y = self.pan_y;

        match self.active_tool {
            ActiveTool::FreeDraw | ActiveTool::Highlighter | ActiveTool::FountainPen | ActiveTool::MagicPen | ActiveTool::LaserPen => {
                self.selected_ids.clear();
                self.current_points.clear();
                self.current_points.push([0.0, 0.0]);
            }
            ActiveTool::Select => {
                let mut clicked_handle = false;
                let mut resizing_id = None;
                
                // First check if we clicked a resize handle of an already selected element
                for &sel_id in &self.selected_ids {
                    if let Some(el) = self.scene.get_element(sel_id) {
                        let handle_size = 16.0 / self.zoom;
                        let w = if el.kind == ElementKind::Text { el.width.abs().max(60.0) } else { el.width };
                        let h = if el.kind == ElementKind::Text { el.height.abs().max(30.0) } else { el.height };
                        if (x - (el.x + w)).abs() < handle_size && (y - (el.y + h)).abs() < handle_size {
                            clicked_handle = true;
                            resizing_id = Some(sel_id);
                            break;
                        }
                    }
                }

                if clicked_handle {
                    self.is_resizing = true;
                    self.drag_start_elements.clear();
                    if let Some(sel_id) = resizing_id {
                        if let Some(el) = self.scene.get_element(sel_id) {
                            self.drag_start_elements.push(el.clone());
                        }
                    }
                } else if let Some(id) = self.scene.hit_test(x, y) {
                    if !self.selected_ids.contains(&id) {
                        self.selected_ids.clear();
                        self.selected_ids.push(id);
                    }
                    
                    self.drag_start_elements.clear();
                    for &sel_id in &self.selected_ids {
                        if let Some(el) = self.scene.get_element(sel_id) {
                            self.drag_start_elements.push(el.clone());
                        }
                    }
                    self.is_resizing = false;
                } else {
                    self.selected_ids.clear();
                    self.is_resizing = false;
                    self.selection_box_start = Some((x, y));
                    self.selection_box_current = Some((x, y));
                }
                self.render();
            }
            ActiveTool::Arrow => {
                if let Some((id, cx, cy)) = self.hit_test_node(x, y) {
                    self.current_start_binding = Some(id);
                    self.start_x = cx;
                    self.start_y = cy;
                } else {
                    self.current_start_binding = None;
                }
            }
            _ => {
                self.selected_ids.clear();
            }
        }
    }

    pub fn on_mouse_move(&mut self, event: &MouseEvent) {
        if !self.is_drawing { return; }
        let (x, y) = self.event_to_canvas(event);
        self.current_x = x;
        self.current_y = y;

        match self.active_tool {
            ActiveTool::FreeDraw | ActiveTool::Highlighter | ActiveTool::FountainPen | ActiveTool::MagicPen | ActiveTool::LaserPen => {
                let dx = x - self.start_x;
                let dy = y - self.start_y;
                self.current_points.push([dx, dy]);
                self.render();
            }
            ActiveTool::Rectangle | ActiveTool::Ellipse | ActiveTool::Line => {
                self.render();
            }
            ActiveTool::Arrow => {
                let mut draw_x = x;
                let mut draw_y = y;
                if let Some((_, cx, cy)) = self.hit_test_node(x, y) {
                    draw_x = cx;
                    draw_y = cy;
                }
                self.current_x = draw_x;
                self.current_y = draw_y;
                self.render();
            }
            ActiveTool::Hand => {
                let rect = self.canvas.get_bounding_client_rect();
                let current_screen_x = event.client_x() as f64 - rect.left();
                let current_screen_y = event.client_y() as f64 - rect.top();
                
                let dx = current_screen_x - self.drag_start_screen_x;
                let dy = current_screen_y - self.drag_start_screen_y;
                
                self.pan_x = self.drag_start_pan_x + dx;
                self.pan_y = self.drag_start_pan_y + dy;
                self.render();
            }
            ActiveTool::Select => {
                if let Some((sx, sy)) = self.selection_box_start {
                    self.selection_box_current = Some((x, y));
                    self.render();
                    
                    // Draw translucent selection box
                    self.ctx.save();
                    self.ctx.translate(self.pan_x, self.pan_y).unwrap();
                    self.ctx.scale(self.zoom, self.zoom).unwrap();
                    self.ctx.set_fill_style_str("rgba(99, 102, 241, 0.1)");
                    self.ctx.set_stroke_style_str("#6366f1");
                    self.ctx.set_line_width(1.0 / self.zoom);
                    self.ctx.fill_rect(sx, sy, x - sx, y - sy);
                    self.ctx.stroke_rect(sx, sy, x - sx, y - sy);
                    self.ctx.restore();
                } else if !self.selected_ids.is_empty() && !self.drag_start_elements.is_empty() {
                    let dx = x - self.start_x;
                    let dy = y - self.start_y;
                    
                    // We need to scale or translate based on drag_start_elements
                    for start_el in &self.drag_start_elements {
                        if let Some(el) = self.scene.get_element_mut(start_el.id) {
                            if self.is_resizing {
                                let w = if start_el.kind == ElementKind::Text { start_el.width.abs().max(60.0) } else { start_el.width };
                                let h = if start_el.kind == ElementKind::Text { start_el.height.abs().max(30.0) } else { start_el.height };
                                
                                let new_w = (w + dx).max(10.0);
                                let new_h = (h + dy).max(10.0);
                                let scale_x = new_w / w;
                                let scale_y = new_h / h;
                                
                                el.width = new_w;
                                el.height = new_h;
                                
                                if start_el.kind == ElementKind::FreeDraw {
                                    el.points = start_el.points.iter().map(|pt| [pt[0] * scale_x, pt[1] * scale_y]).collect();
                                } else if start_el.kind == ElementKind::Text {
                                    el.height = new_h; // Update font size
                                }
                            } else {
                                el.x = start_el.x + dx;
                                el.y = start_el.y + dy;
                            }
                            self.is_dirty = true;
                        }
                    }
                    self.render();
                }
            }
            ActiveTool::Eraser => {
                // Erase any element whose bounding box the pointer is inside
                let eraser_radius = (self.eraser_size / 2.0).max(12.0);
                let ids_to_remove: Vec<u64> = self.scene.elements
                    .iter()
                    .filter(|el| el.contains_radius(x, y, eraser_radius))
                    .map(|el| el.id)
                    .collect();
                if !ids_to_remove.is_empty() {
                    let mut txn = self.doc.transact_mut();
                    for id in &ids_to_remove {
                        self.elements_map.remove(&mut txn, id.to_string().as_str());
                    }
                    self.scene.elements.retain(|el| !ids_to_remove.contains(&el.id));
                    self.is_dirty = true;
                    self.render();
                }
            }
            _ => {}
        }
    }

    pub fn on_mouse_up(&mut self, event: &MouseEvent) {
        if !self.is_drawing { return; }
        self.is_drawing = false;
        let (x, y) = self.event_to_canvas(event);

        match self.active_tool {
            ActiveTool::FreeDraw | ActiveTool::Highlighter | ActiveTool::FountainPen | ActiveTool::MagicPen | ActiveTool::LaserPen => {
                if self.current_points.len() > 1 {
                    let id = self.scene.next_id();
                    let mut el = SceneElement::new_free_draw(
                        id,
                        self.start_x,
                        self.start_y,
                        self.current_points.clone(),
                    );
                    el.stroke = self.stroke_color.clone();
                    el.stroke_width = self.stroke_width;
                    if self.active_tool == ActiveTool::Highlighter {
                        el.is_highlighter = true;
                        el.stroke_width = self.stroke_width.max(12.0);
                    } else if self.active_tool == ActiveTool::FountainPen {
                        el.is_fountain_pen = true;
                        el.fountain_sharpness = self.fountain_sharpness;
                    } else if self.active_tool == ActiveTool::MagicPen {
                        el.stroke = "#6366f1".to_string(); // Magic pen is always indigo so you know it's working
                        el.stroke_width = 3.0;
                        self.magic_stroke_ids.push(id);
                    } else if self.active_tool == ActiveTool::LaserPen {
                        el.is_laser = true;
                        el.stroke = self.stroke_color.clone(); // Dynamic laser color
                        el.stroke_width = self.stroke_width.max(6.0);
                        el.alpha = Some(1.0);
                    }
                    self.scene.add(el);
                    
                    // Immediately sync magic stroke to CRDT so it doesn't get lost before extraction
                    let mut txn = self.doc.transact_mut();
                    if let Some(el_ref) = self.scene.elements.last() {
                        if let Ok(json_str) = serde_json::to_string(el_ref) {
                            self.elements_map.insert(&mut txn, id.to_string(), json_str);
                            self.is_dirty = true;
                        }
                    }
                }
                self.current_points.clear();
            }
            ActiveTool::Rectangle => {
                let (rx, ry, rw, rh) = Self::normalize_rect(self.start_x, self.start_y, x, y);
                if rw > 2.0 && rh > 2.0 {
                    let mut el = SceneElement::new_rect(self.scene.next_id(), rx, ry, rw, rh);
                    el.stroke = self.stroke_color.clone();
                    el.fill = self.fill_color.clone();
                    el.stroke_width = self.stroke_width;
                    el.is_rough = self.is_rough;
                    self.scene.add(el);
                }
            }
            ActiveTool::Ellipse => {
                let (rx, ry, rw, rh) = Self::normalize_rect(self.start_x, self.start_y, x, y);
                if rw > 2.0 && rh > 2.0 {
                    let mut el = SceneElement::new_rect(self.scene.next_id(), rx, ry, rw, rh);
                    el.kind = ElementKind::Ellipse;
                    el.stroke = self.stroke_color.clone();
                    el.fill = self.fill_color.clone();
                    el.stroke_width = self.stroke_width;
                    el.is_rough = self.is_rough;
                    self.scene.add(el);
                }
            }
            ActiveTool::Line => {
                let mut el = SceneElement::new_rect(self.scene.next_id(), self.start_x, self.start_y, x - self.start_x, y - self.start_y);
                el.kind = ElementKind::Line;
                el.stroke = self.stroke_color.clone();
                el.stroke_width = self.stroke_width;
                self.scene.add(el);
            }
            ActiveTool::Arrow => {
                let mut end_x = x;
                let mut end_y = y;
                if let Some((id, cx, cy)) = self.hit_test_node(x, y) {
                    self.current_end_binding = Some(id);
                    end_x = cx;
                    end_y = cy;
                } else {
                    self.current_end_binding = None;
                }
                
                let mut el = SceneElement::new_rect(self.scene.next_id(), self.start_x, self.start_y, end_x - self.start_x, end_y - self.start_y);
                el.kind = ElementKind::Arrow;
                el.stroke = self.stroke_color.clone();
                el.stroke_width = self.stroke_width;
                el.is_curved = self.is_curved;
                el.start_binding = self.current_start_binding;
                el.end_binding = self.current_end_binding;
                self.scene.add(el);
            }
            ActiveTool::Select => {
                if let Some((sx, sy)) = self.selection_box_start {
                    let mut new_selection = Vec::new();
                    let rx = sx.min(x);
                    let ry = sy.min(y);
                    let rw = (x - sx).abs();
                    let rh = (y - sy).abs();
                    
                    if rw > 2.0 && rh > 2.0 {
                        for el in &self.scene.elements {
                            if el.intersects_rect(rx, ry, rw, rh) {
                                new_selection.push(el.id);
                            }
                        }
                    }
                    
                    self.selected_ids = new_selection;
                    self.selection_box_start = None;
                    self.selection_box_current = None;
                }
            }
            _ => {}
        }

        self.is_dirty = true;
        self.render();
    }

    // ── Core render loop ───────────────────────────────────────────────────

    /// Full scene repaint — called after every committed action.
    pub fn render(&self) {
        let w = self.canvas.width() as f64;
        let h = self.canvas.height() as f64;

        // Clear
        self.ctx.clear_rect(0.0, 0.0, w, h);

        // Background
        let bg = if self.dark_mode { "#000000" } else { BG_COLOR };
        self.ctx.set_fill_style_str(bg);
        self.ctx.fill_rect(0.0, 0.0, w, h);

        // Apply pan + zoom transform
        self.ctx.save();
        self.ctx.translate(self.pan_x, self.pan_y).unwrap();
        self.ctx.scale(self.zoom, self.zoom).unwrap();

        // Grid
        self.draw_grid(w, h);

        // Scene elements
        for el in &self.scene.elements {
            self.draw_element(el);
        }

        // Selection outline
        if !self.selected_ids.is_empty() {
            for &id in &self.selected_ids {
                if let Some(el) = self.scene.get_element(id) {
                    self.draw_selection_box(el);
                }
            }
        }
        
        // Always draw live elements on top to prevent flicker during animations
        if self.is_drawing {
            let cx = self.current_x;
            let cy = self.current_y;
            match self.active_tool {
                ActiveTool::FreeDraw | ActiveTool::Highlighter | ActiveTool::FountainPen | ActiveTool::MagicPen | ActiveTool::LaserPen => {
                    self.draw_live_freedraw();
                }
                ActiveTool::Rectangle => self.draw_live_rect(cx, cy),
                ActiveTool::Ellipse => self.draw_live_ellipse(cx, cy),
                ActiveTool::Line => self.draw_live_line(cx, cy),
                ActiveTool::Arrow => self.draw_live_arrow(cx, cy),
                _ => {}
            }
        }

        self.ctx.restore();
    }

    // ── External Element insertion (Images) ────────────────────────────────

    pub fn add_image(&mut self, img: HtmlImageElement, x: f64, y: f64, w: f64, h: f64, asset_id: &str) {
        let id = self.scene.next_id();
        let mut el = SceneElement::new_rect(id, x, y, w, h);
        el.kind = ElementKind::Image;
        el.asset_id = Some(asset_id.to_string());
        el.stroke = "transparent".to_string(); // no border by default
        self.scene.add(el);
        self.is_dirty = true;
        self.image_cache.insert(id, img);
        self.render();
    }

    pub fn add_diagram(&mut self, img: HtmlImageElement, x: f64, y: f64, w: f64, h: f64, dsl_code: &str, svg_data: &str, hit_map_json: &str) {
        let id = self.scene.next_id();
        let mut el = SceneElement::new_rect(id, x, y, w, h);
        el.kind = ElementKind::Diagram;
        el.text = Some(dsl_code.to_string()); // store the raw ArasDiagram DSL
        el.asset_id = Some(svg_data.to_string());
        if let Ok(hit_map) = serde_json::from_str(hit_map_json) {
            el.hit_map = Some(hit_map);
        }
        el.stroke = "transparent".to_string(); // no border by default
        self.scene.add(el);
        self.is_dirty = true;
        self.image_cache.insert(id, img);
        self.render();
    }


    pub fn set_cached_image(&mut self, id: u64, img: HtmlImageElement) {
        self.image_cache.insert(id, img);
        self.render();
    }

    pub fn on_double_click(&mut self, event: &MouseEvent) -> Option<String> {
        let (x, y) = self.event_to_canvas(event);
        
        // Find if we clicked on a Diagram
        for el in self.scene.elements.iter().rev() {
            if el.kind == ElementKind::Diagram {
                if el.contains(x, y) {
                    if let Some(hit_map) = &el.hit_map {
                        // Calculate local coordinates relative to the diagram
                        let local_x = x - el.x;
                        let local_y = y - el.y;
                        
                        // Check hit map
                        for (node_id, (nx, ny, nw, nh)) in hit_map {
                            if local_x >= *nx && local_x <= *nx + *nw &&
                               local_y >= *ny && local_y <= *ny + *nh {
                                return Some(format!("{},{}", el.id, node_id));
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    pub fn get_element_code(&self, id: u64) -> Option<String> {
        self.scene.elements.iter().find(|el| el.id == id).and_then(|el| el.text.clone())
    }

    // ── Internal helpers ───────────────────────────────────────────────────

    /// Inverts black/dark colors to white in dark mode.
    fn get_adaptive_color(&self, color: &str) -> String {
        if self.dark_mode && (color == "#1a1a2e" || color == "#000000" || color == "#111111") {
            "#FFFFFF".to_string()
        } else {
            color.to_string()
        }
    }

    fn event_to_canvas(&self, event: &MouseEvent) -> (f64, f64) {
        let rect = self.canvas.get_bounding_client_rect();
        let raw_x = event.client_x() as f64 - rect.left();
        let raw_y = event.client_y() as f64 - rect.top();
        // Convert screen coords → world coords
        ((raw_x - self.pan_x) / self.zoom, (raw_y - self.pan_y) / self.zoom)
    }

    fn draw_grid(&self, viewport_w: f64, viewport_h: f64) {
        if self.grid_type == "blank" { return; }

        // Compute world-space bounds of the visible viewport
        let world_start_x = -self.pan_x / self.zoom;
        let world_start_y = -self.pan_y / self.zoom;
        let world_end_x = world_start_x + viewport_w / self.zoom;
        let world_end_y = world_start_y + viewport_h / self.zoom;

        // First grid line to the left/top of the visible area
        let first_x = (world_start_x / GRID_SPACING).floor() * GRID_SPACING;
        let first_y = (world_start_y / GRID_SPACING).floor() * GRID_SPACING;

        let grid_color = if self.dark_mode { "rgba(255,255,255,0.05)" } else { GRID_DOT_COLOR };
        
        self.ctx.set_fill_style_str(grid_color);
        self.ctx.set_stroke_style_str(grid_color);
        self.ctx.set_line_width(1.0);

        if self.grid_type == "lines" {
            let dash_array = js_sys::Array::new();
            dash_array.push(&JsValue::from_f64(2.0));
            dash_array.push(&JsValue::from_f64(4.0));
            self.ctx.set_line_dash(&dash_array).unwrap();
            
            self.ctx.begin_path();
            let mut gx = first_x;
            while gx <= world_end_x {
                self.ctx.move_to(gx, world_start_y);
                self.ctx.line_to(gx, world_end_y);
                gx += GRID_SPACING;
            }
            let mut gy = first_y;
            while gy <= world_end_y {
                self.ctx.move_to(world_start_x, gy);
                self.ctx.line_to(world_end_x, gy);
                gy += GRID_SPACING;
            }
            self.ctx.stroke();
            self.ctx.set_line_dash(&js_sys::Array::new()).unwrap();
        } else { // "dots"
            let mut gx = first_x;
            while gx <= world_end_x {
                let mut gy = first_y;
                while gy <= world_end_y {
                    self.ctx.begin_path();
                    self.ctx.arc(gx, gy, 0.5, 0.0, std::f64::consts::TAU).unwrap();
                    self.ctx.fill();
                    gy += GRID_SPACING;
                }
                gx += GRID_SPACING;
            }
        }
    }

    fn draw_element(&self, el: &SceneElement) {
        self.ctx.set_stroke_style_str(&self.get_adaptive_color(&el.stroke));
        self.ctx.set_fill_style_str(&self.get_adaptive_color(&el.fill));
        self.ctx.set_line_width(el.stroke_width);
        self.ctx.set_line_cap("round");
        self.ctx.set_line_join("round");

        if el.is_highlighter {
            let blend = if self.dark_mode { "screen" } else { "multiply" };
            self.ctx.set_global_composite_operation(blend).unwrap();
            self.ctx.set_global_alpha(0.35); // 35% opacity
        } else {
            self.ctx.set_global_composite_operation("source-over").unwrap();
            if let Some(alpha) = el.alpha {
                self.ctx.set_global_alpha(alpha);
            } else {
                self.ctx.set_global_alpha(1.0);
            }
        }

        match el.kind {
            ElementKind::Rectangle => {
                self.ctx.begin_path();
                if el.is_rough {
                    let mut seed = el.id.wrapping_add(1).wrapping_mul(123456789);
                    let mut next_rand = || -> f64 {
                        seed ^= seed << 13;
                        seed ^= seed >> 17;
                        seed ^= seed << 5;
                        (seed % 1000) as f64 / 1000.0 * 2.0 - 1.0 // -1.0 to 1.0
                    };
                    for _ in 0..2 { // Double stroke for scribble effect
                        let max_offset = 3.0;
                        self.ctx.move_to(el.x + next_rand() * max_offset, el.y + next_rand() * max_offset);
                        self.ctx.line_to(el.x + el.width + next_rand() * max_offset, el.y + next_rand() * max_offset);
                        
                        self.ctx.move_to(el.x + el.width + next_rand() * max_offset, el.y + next_rand() * max_offset);
                        self.ctx.line_to(el.x + el.width + next_rand() * max_offset, el.y + el.height + next_rand() * max_offset);
                        
                        self.ctx.move_to(el.x + el.width + next_rand() * max_offset, el.y + el.height + next_rand() * max_offset);
                        self.ctx.line_to(el.x + next_rand() * max_offset, el.y + el.height + next_rand() * max_offset);
                        
                        self.ctx.move_to(el.x + next_rand() * max_offset, el.y + el.height + next_rand() * max_offset);
                        self.ctx.line_to(el.x + next_rand() * max_offset, el.y + next_rand() * max_offset);
                    }
                } else {
                    self.ctx.rect(el.x, el.y, el.width, el.height);
                }
                if el.fill != "transparent" { self.ctx.fill(); }
                if el.stroke != "transparent" { self.ctx.stroke(); }
            }
            ElementKind::Ellipse => {
                self.ctx.begin_path();
                let cx = el.x + el.width / 2.0;
                let cy = el.y + el.height / 2.0;
                let rx = el.width.abs() / 2.0;
                let ry = el.height.abs() / 2.0;
                if el.is_rough {
                    let mut seed = el.id.wrapping_add(2).wrapping_mul(987654321);
                    let mut next_rand = || -> f64 {
                        seed ^= seed << 13;
                        seed ^= seed >> 17;
                        seed ^= seed << 5;
                        (seed % 1000) as f64 / 1000.0 * 2.0 - 1.0 // -1.0 to 1.0
                    };
                    for _ in 0..2 {
                        let mut first = true;
                        let steps = 16;
                        for i in 0..=steps {
                            let angle = (i as f64 / steps as f64) * std::f64::consts::TAU;
                            let max_offset = 3.0;
                            let px = cx + rx * angle.cos() + next_rand() * max_offset;
                            let py = cy + ry * angle.sin() + next_rand() * max_offset;
                            if first {
                                self.ctx.move_to(px, py);
                                first = false;
                            } else {
                                self.ctx.line_to(px, py);
                            }
                        }
                    }
                } else {
                    self.ctx.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::TAU).unwrap();
                }
                if el.fill != "transparent" { self.ctx.fill(); }
                if el.stroke != "transparent" { self.ctx.stroke(); }
            }
            ElementKind::Line => {
                self.ctx.begin_path();
                self.ctx.move_to(el.x, el.y);
                self.ctx.line_to(el.x + el.width, el.y + el.height);
                if el.stroke != "transparent" { self.ctx.stroke(); }
            }
            ElementKind::FreeDraw => {
                if el.points.len() < 2 { return; }
                
                let draw_path = |ctx: &CanvasRenderingContext2d| {
                    ctx.begin_path();
                    ctx.move_to(el.x + el.points[0][0], el.y + el.points[0][1]);
                    
                    let len = el.points.len();
                    if len < 3 {
                        for pt in el.points.iter().skip(1) {
                            ctx.line_to(el.x + pt[0], el.y + pt[1]);
                        }
                    } else {
                        for i in 1..(len - 2) {
                            let xc = (el.points[i][0] + el.points[i + 1][0]) / 2.0;
                            let yc = (el.points[i][1] + el.points[i + 1][1]) / 2.0;
                            ctx.quadratic_curve_to(
                                el.x + el.points[i][0], el.y + el.points[i][1],
                                el.x + xc, el.y + yc
                            );
                        }
                        ctx.quadratic_curve_to(
                            el.x + el.points[len - 2][0], el.y + el.points[len - 2][1],
                            el.x + el.points[len - 1][0], el.y + el.points[len - 1][1]
                        );
                    }
                };

                if el.is_fountain_pen {
                    // Draw multiple times with offset for calligraphy broad nib effect
                    // Nib angle = 45 degrees
                    let cos_a = std::f64::consts::FRAC_PI_4.cos();
                    let sin_a = std::f64::consts::FRAC_PI_4.sin();
                    let strokes = (el.stroke_width * 2.0) as i32;
                    self.ctx.set_line_width(0.5); // very thin base stroke
                    
                    for i in -strokes/2..=strokes/2 {
                        let offset = i as f64 * el.fountain_sharpness;
                        let dx = offset * cos_a;
                        let dy = offset * sin_a;
                        self.ctx.save();
                        self.ctx.translate(dx, dy).unwrap();
                        draw_path(&self.ctx);
                        if el.stroke != "transparent" { self.ctx.stroke(); }
                        self.ctx.restore();
                    }
                } else if el.is_laser {
                    self.ctx.save();
                    self.ctx.set_shadow_blur(20.0);
                    let adapt = self.get_adaptive_color(&el.stroke);
                    self.ctx.set_shadow_color(&adapt);
                    self.ctx.set_line_width(el.stroke_width * 1.5);
                    self.ctx.set_stroke_style_str(&adapt);
                    draw_path(&self.ctx);
                    if el.stroke != "transparent" { self.ctx.stroke(); }
                    
                    // Draw a softer core
                    self.ctx.set_shadow_blur(0.0);
                    self.ctx.set_line_width(el.stroke_width * 0.5);
                    self.ctx.set_stroke_style_str("#ffffff");
                    draw_path(&self.ctx);
                    self.ctx.stroke();
                    self.ctx.restore();
                } else {
                    draw_path(&self.ctx);
                    if el.stroke != "transparent" { self.ctx.stroke(); }
                }
            }
            ElementKind::Arrow => {
                let mut sx = el.x;
                let mut sy = el.y;
                let mut ex = el.x + el.width;
                let mut ey = el.y + el.height;
                
                if let Some(bid) = el.start_binding {
                    if let Some(bind_el) = self.scene.get_element(bid) {
                        sx = bind_el.x + bind_el.width / 2.0;
                        sy = bind_el.y + bind_el.height / 2.0;
                    }
                }
                if let Some(bid) = el.end_binding {
                    if let Some(bind_el) = self.scene.get_element(bid) {
                        ex = bind_el.x + bind_el.width / 2.0;
                        ey = bind_el.y + bind_el.height / 2.0;
                    }
                }

                self.ctx.begin_path();
                self.ctx.move_to(sx, sy);
                
                let mut angle = (ey - sy).atan2(ex - sx);
                
                if el.is_curved {
                    // Draw bezier curve dipping slightly in the middle
                    let mid_x = (sx + ex) / 2.0;
                    let mid_y = (sy + ey) / 2.0;
                    let dist = ((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt();
                    let dip = dist * 0.2;
                    let perp_x = -(ey - sy) / dist * dip;
                    let perp_y = (ex - sx) / dist * dip;
                    let cp_x = mid_x + perp_x;
                    let cp_y = mid_y + perp_y;
                    
                    self.ctx.quadratic_curve_to(cp_x, cp_y, ex, ey);
                    
                    // compute angle for arrowhead based on control point to end
                    angle = (ey - cp_y).atan2(ex - cp_x);
                } else {
                    self.ctx.line_to(ex, ey);
                }
                
                if el.stroke != "transparent" { self.ctx.stroke(); }
                
                let head_len = 15.0;
                let p1_x = ex - head_len * (angle - std::f64::consts::PI / 6.0).cos();
                let p1_y = ey - head_len * (angle - std::f64::consts::PI / 6.0).sin();
                let p2_x = ex - head_len * (angle + std::f64::consts::PI / 6.0).cos();
                let p2_y = ey - head_len * (angle + std::f64::consts::PI / 6.0).sin();
                
                self.ctx.begin_path();
                self.ctx.move_to(ex, ey);
                self.ctx.line_to(p1_x, p1_y);
                self.ctx.move_to(ex, ey);
                self.ctx.line_to(p2_x, p2_y);
                
                if el.stroke != "transparent" { self.ctx.stroke(); }
            }
            ElementKind::Image | ElementKind::Diagram => {
                let raster_cache = self.raster_cache.borrow();
                if let Some(canvas) = raster_cache.get(&el.id) {
                    self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                        canvas, el.x, el.y, el.width, el.height
                    ).unwrap();
                } else {
                    drop(raster_cache); // Release borrow before mutating
                    
                    if let Some(img) = self.image_cache.get(&el.id) {
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        if let Ok(canvas_el) = document.create_element("canvas") {
                            if let Ok(raster_canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                                let nw = img.natural_width();
                                let nh = img.natural_height();
                                
                                if nw > 0 && nh > 0 {
                                    raster_canvas.set_width(nw);
                                    raster_canvas.set_height(nh);
                                    
                                    if let Ok(Some(rctx_obj)) = raster_canvas.get_context("2d") {
                                        if let Ok(rctx) = rctx_obj.dyn_into::<CanvasRenderingContext2d>() {
                                            let _ = rctx.draw_image_with_html_image_element(img, 0.0, 0.0);
                                            
                                            self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                                                &raster_canvas, el.x, el.y, el.width, el.height
                                            ).unwrap();
                                            
                                            self.raster_cache.borrow_mut().insert(el.id, raster_canvas);
                                        }
                                    }
                                } else {
                                    let _ = self.ctx.draw_image_with_html_image_element_and_dw_and_dh(
                                        img, el.x, el.y, el.width, el.height
                                    );
                                }
                            }
                        }
                    } else {
                        self.ctx.begin_path();
                        self.ctx.rect(el.x, el.y, el.width, el.height);
                        self.ctx.set_fill_style_str("#e5e7eb");
                        self.ctx.fill();
                    }
                }
            }
            ElementKind::Text => {
                if let Some(ref text) = el.text {
                    let default_font = "'Space Grotesk', sans-serif".to_string();
                    let font_name = el.font_family.as_ref().unwrap_or(&default_font);
                    self.ctx.set_font(&format!("{}px {}", el.height, font_name));
                    self.ctx.set_text_baseline("top");
                    self.ctx.set_fill_style_str(&self.get_adaptive_color(&el.stroke));
                    
                    let lines: Vec<&str> = text.split('\n').collect();
                    let line_height = el.height * 1.2; // 1.2em line height
                    
                    for (i, line) in lines.iter().enumerate() {
                        let y_offset = el.y + (i as f64 * line_height);
                        self.ctx.fill_text(line, el.x, y_offset).unwrap();
                    }
                }
            }
        }

        // Ensure we reset blend modes after drawing
        self.ctx.set_global_composite_operation("source-over").unwrap();
        self.ctx.set_global_alpha(1.0);
    }

    fn draw_selection_box(&self, el: &SceneElement) {
        let padding = (el.stroke_width / 2.0).max(4.0) + 4.0;
        
        let (mut bx, mut by, mut bw, mut bh) = match el.kind {
            ElementKind::Rectangle | ElementKind::Ellipse | ElementKind::Line | ElementKind::Arrow | ElementKind::Image | ElementKind::Diagram => {
                let x = el.x.min(el.x + el.width);
                let y = el.y.min(el.y + el.height);
                let w = el.width.abs();
                let h = el.height.abs();
                (x, y, w, h)
            },
            ElementKind::FreeDraw => {
                if el.points.is_empty() { return; }
                let mut min_x = 0.0;
                let mut max_x = 0.0;
                let mut min_y = 0.0;
                let mut max_y = 0.0;
                for pt in &el.points {
                    if pt[0] < min_x { min_x = pt[0]; }
                    if pt[0] > max_x { max_x = pt[0]; }
                    if pt[1] < min_y { min_y = pt[1]; }
                    if pt[1] > max_y { max_y = pt[1]; }
                }
                (el.x + min_x, el.y + min_y, max_x - min_x, max_y - min_y)
            },
            ElementKind::Text => (el.x, el.y, el.width, el.height)
        };
        
        bx -= padding;
        by -= padding;
        bw += padding * 2.0;
        bh += padding * 2.0;

        self.ctx.set_stroke_style_str("#6366f1");
        self.ctx.set_line_width(2.0);
        let dashes = js_sys::Array::of2(&JsValue::from_f64(4.0), &JsValue::from_f64(4.0));
        self.ctx.set_line_dash(&dashes).unwrap();
        self.ctx.stroke_rect(bx, by, bw, bh);
        self.ctx.set_line_dash(&js_sys::Array::new()).unwrap();
        
        // Draw resize handle at bottom right
        self.ctx.set_fill_style_str("#ffffff");
        self.ctx.set_stroke_style_str("#6366f1");
        let handle_size = 10.0 / self.zoom;
        self.ctx.fill_rect(bx + bw - handle_size / 2.0, by + bh - handle_size / 2.0, handle_size, handle_size);
        self.ctx.stroke_rect(bx + bw - handle_size / 2.0, by + bh - handle_size / 2.0, handle_size, handle_size);
    }

    fn draw_live_freedraw(&self) {
        if self.current_points.len() < 2 { return; }
        self.ctx.save();
        self.ctx.set_stroke_style_str(&self.get_adaptive_color(&self.stroke_color));
        self.ctx.set_line_width(self.stroke_width);
        self.ctx.set_line_cap("round");
        self.ctx.set_line_join("round");
        
        if self.active_tool == ActiveTool::Highlighter {
            let blend = if self.dark_mode { "screen" } else { "multiply" };
            self.ctx.set_global_composite_operation(blend).unwrap();
            self.ctx.set_global_alpha(0.35);
            self.ctx.set_line_width(self.stroke_width.max(12.0));
        } else if self.active_tool == ActiveTool::MagicPen {
            self.ctx.set_stroke_style_str("#6366f1");
            self.ctx.set_line_width(3.0);
        } else if self.active_tool == ActiveTool::LaserPen {
            self.ctx.set_shadow_blur(15.0);
            let adapt = self.get_adaptive_color(&self.stroke_color);
            self.ctx.set_shadow_color(&adapt);
            self.ctx.set_stroke_style_str(&adapt);
            self.ctx.set_line_width(self.stroke_width.max(6.0));
        }
        
        let draw_path = |ctx: &CanvasRenderingContext2d| {
            ctx.begin_path();
            let sx = self.start_x;
            let sy = self.start_y;
            ctx.move_to(sx + self.current_points[0][0], sy + self.current_points[0][1]);
            
            let len = self.current_points.len();
            if len < 3 {
                for pt in self.current_points.iter().skip(1) {
                    ctx.line_to(sx + pt[0], sy + pt[1]);
                }
            } else {
                for i in 1..(len - 2) {
                    let xc = (self.current_points[i][0] + self.current_points[i + 1][0]) / 2.0;
                    let yc = (self.current_points[i][1] + self.current_points[i + 1][1]) / 2.0;
                    ctx.quadratic_curve_to(
                        sx + self.current_points[i][0], sy + self.current_points[i][1],
                        sx + xc, sy + yc
                    );
                }
                ctx.quadratic_curve_to(
                    sx + self.current_points[len - 2][0], sy + self.current_points[len - 2][1],
                    sx + self.current_points[len - 1][0], sy + self.current_points[len - 1][1]
                );
            }
        };

        if self.active_tool == ActiveTool::FountainPen {
            let cos_a = std::f64::consts::FRAC_PI_4.cos();
            let sin_a = std::f64::consts::FRAC_PI_4.sin();
            let strokes = (self.stroke_width * 2.0) as i32;
            self.ctx.set_line_width(0.5);
            for i in -strokes/2..=strokes/2 {
                let offset = i as f64 * self.fountain_sharpness;
                self.ctx.save();
                self.ctx.translate(offset * cos_a, offset * sin_a).unwrap();
                draw_path(&self.ctx);
                self.ctx.stroke();
                self.ctx.restore();
            }
        } else if self.active_tool == ActiveTool::LaserPen {
            self.ctx.save();
            draw_path(&self.ctx);
            self.ctx.stroke();
            
            // Draw a softer core
            self.ctx.set_shadow_blur(0.0);
            self.ctx.set_line_width(self.stroke_width.max(6.0) * 0.5);
            self.ctx.set_stroke_style_str("#ffffff");
            draw_path(&self.ctx);
            self.ctx.stroke();
            self.ctx.restore();
        } else {
            draw_path(&self.ctx);
            self.ctx.stroke();
        }
        
        self.ctx.restore();
    }

    fn draw_live_rect(&self, x: f64, y: f64) {
        let (rx, ry, rw, rh) = Self::normalize_rect(self.start_x, self.start_y, x, y);
        self.ctx.save();
        self.ctx.set_stroke_style_str(&self.get_adaptive_color(&self.stroke_color));
        self.ctx.set_line_width(self.stroke_width);
        // Dashed preview
        let dashes = js_sys::Array::of2(&JsValue::from_f64(6.0), &JsValue::from_f64(4.0));
        self.ctx.set_line_dash(&dashes).unwrap();
        self.ctx.stroke_rect(rx, ry, rw, rh);
        self.ctx.set_line_dash(&js_sys::Array::new()).unwrap();
        self.ctx.restore();
    }

    fn draw_live_ellipse(&self, x: f64, y: f64) {
        let (rx, ry, rw, rh) = Self::normalize_rect(self.start_x, self.start_y, x, y);
        let cx = rx + rw / 2.0;
        let cy = ry + rh / 2.0;
        self.ctx.save();
        self.ctx.set_stroke_style_str(&self.get_adaptive_color(&self.stroke_color));
        self.ctx.set_line_width(self.stroke_width);
        let dashes = js_sys::Array::of2(&JsValue::from_f64(6.0), &JsValue::from_f64(4.0));
        self.ctx.set_line_dash(&dashes).unwrap();
        self.ctx.begin_path();
        self.ctx.ellipse(cx, cy, rw / 2.0, rh / 2.0, 0.0, 0.0, std::f64::consts::TAU).unwrap();
        self.ctx.stroke();
        self.ctx.set_line_dash(&js_sys::Array::new()).unwrap();
        self.ctx.restore();
    }

    fn draw_live_line(&self, x: f64, y: f64) {
        self.ctx.save();
        self.ctx.set_stroke_style_str(&self.get_adaptive_color(&self.stroke_color));
        self.ctx.set_line_width(self.stroke_width);
        self.ctx.set_line_cap("round");
        self.ctx.begin_path();
        self.ctx.move_to(self.start_x, self.start_y);
        self.ctx.line_to(x, y);
        self.ctx.stroke();
        self.ctx.restore();
    }

    fn draw_live_arrow(&self, current_x: f64, current_y: f64) {
        let mut el = SceneElement::new_rect(0, self.start_x, self.start_y, current_x - self.start_x, current_y - self.start_y);
        el.kind = ElementKind::Arrow;
        el.stroke = self.stroke_color.clone();
        el.stroke_width = self.stroke_width;
        el.is_curved = self.is_curved;
        self.draw_element(&el);
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if self.selected_ids.len() == 1 {
            let id = self.selected_ids[0];
            if let Some(el) = self.scene.elements.iter().find(|e| e.id == id) {
                if el.kind == ElementKind::Text {
                    return el.text.clone();
                }
            }
        }
        None
    }

    pub fn update_selected_text(&mut self, text: &str) {
        if self.selected_ids.len() == 1 {
            let id = self.selected_ids[0];
            if let Some(el) = self.scene.get_element_mut(id) {
                if el.kind == ElementKind::Text {
                    el.text = Some(text.to_string());
                    self.is_dirty = true;
                    
                    
                    let clone = el.clone();
                    let mut txn = self.doc.transact_mut();
                    if let Ok(json_str) = serde_json::to_string(&clone) {
                        self.elements_map.insert(&mut txn, id.to_string().as_str(), json_str);
                    }
                    
                    self.render();
                }
            }
        }
    }



    fn normalize_rect(x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64, f64, f64) {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let w = (x2 - x1).abs();
        let h = (y2 - y1).abs();
        (x, y, w, h)
    }

    fn hit_test_node(&self, x: f64, y: f64) -> Option<(u64, f64, f64)> {
        for el in self.scene.elements.iter().rev() {
            if (el.kind == ElementKind::Rectangle || el.kind == ElementKind::Ellipse || el.kind == ElementKind::Image) && el.contains(x, y) {
                return Some((el.id, el.x + el.width / 2.0, el.y + el.height / 2.0));
            }
        }
        None
    }
}
