# Custom Rust WebAssembly Rendering Engine

You are making a bold and highly strategic move! Building a proprietary, high-performance rendering engine in Rust (compiled to WebAssembly) gives you absolute control, blistering performance, and a unique, copyrightable core architecture that differentiates **Aerial** from just being a wrapper around open-source tools.

This plan outlines the monumental (but incredibly exciting) first step: stripping out Excalidraw and scaffolding your own Wasm-powered rendering pipeline.

## User Review Required

> [!CAUTION]  
> This means we will no longer use Excalidraw. We will be building the canvas interactions (drawing shapes, moving, selecting) from scratch in Rust and WebAssembly. 
> 
> The initial implementation will just prove the pipeline works by drawing basic shapes directly to the HTML5 Canvas from Rust. 

## Open Questions

> [!WARNING]  
> 1. To compile Rust into WebAssembly, we will need to install a tool called `wasm-pack`. I will run `cargo install wasm-pack` during execution. Are you okay with this?
> 2. WebAssembly rendering can be done using standard 2D Canvas context, or WebGL/WebGPU for maximum performance. To start, I recommend the standard 2D context via the `web-sys` crate to get shapes on the board quickly. Does that work for you?

## Proposed Changes

### 1. Strip out Excalidraw
- Remove `@excalidraw/excalidraw` from `package.json`.
- Strip the `<Excalidraw />` component from `App.tsx`.
- Replace the main area with a raw HTML5 `<canvas id="aerial-canvas" />`.

### 2. Scaffold `aerial-engine` (Rust Wasm Crate)
- Create a new Rust library crate inside the project called `aerial-engine`.
- Add dependencies: `wasm-bindgen` and `web-sys` (with Canvas features enabled).
- Write the initial Rust engine code that attaches to the canvas element and exposes a `render()` function to JavaScript.
- Implement a basic drawing function (e.g., drawing a smooth signature or bone-white shapes) to prove the bridge is working.

### 3. Connect React to WebAssembly
- Configure Vite to support WebAssembly imports.
- Update `App.tsx` to asynchronously load the compiled `aerial-engine` Wasm module.
- Pass the canvas reference from React into the Rust Wasm engine.

## Verification Plan

### Automated Tests
- Run `wasm-pack build --target web` in the `aerial-engine` directory.
- Run `npm run build` for the Vite frontend.

### Manual Verification
- Start the app using `npm run tauri dev`.
- We will see a custom canvas area where a shape or line has been drawn directly by our proprietary Rust engine, completely bypassing standard JavaScript rendering libraries!
