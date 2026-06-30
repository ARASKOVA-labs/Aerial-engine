/*!
 * aerial-engine
 * Proprietary Rust WebAssembly 2D Rendering Engine
 * © ARASKOVA Labs — All rights reserved.
 *
 * This is the core rendering pipeline for Aerial Board Software.
 * All canvas interactions, scene management, and tool behaviors are
 * authored, compiled, and owned by ARASKOVA Labs.
 */

mod canvas;
mod scene;
mod tools;
mod utils;

use wasm_bindgen::prelude::*;

// Called once by the JS bootstrap to wire up panic hook for debugging
#[wasm_bindgen(start)]
pub fn init() {
    utils::set_panic_hook();
}

// Re-export the public API surface
pub use canvas::AerialCanvas;
