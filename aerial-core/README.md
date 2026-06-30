<div align="center">
  <img src="https://raw.githubusercontent.com/ARASKOVA-labs/Aerial-engine/main/.github/A-monogram.svg" alt="A Monogram" width="120" />
  <h1>Aerial Engine</h1>
  <p><strong>A bare-metal, zero-copy CRDT canvas engine built in pure Rust.</strong></p>
  <p>120 FPS • Local-First • Offline Capable • Built for Scale</p>
  <br />
  <img src="https://via.placeholder.com/800x400.png?text=Demo+Video+Placeholder:+Aerial+at+10,000+strokes" alt="Aerial Demo Video" />
</div>

---

## ⚡ Why Aerial Engine?

Most web-based whiteboard and canvas applications hit a hard memory and garbage collection ceiling after a few thousand strokes. To scale beyond that, you can't just write faster JavaScript. You need a different architecture.

**Aerial Engine** is a WASM-first engine. It completely bypasses V8 garbage collection by managing all layout, strokes, and scene structures in a pure Rust WebAssembly (`wasm32-unknown-unknown`) environment.

### 🏗️ Architecture

- **Zero-Copy CRDTs**: Powered by `yrs` (Yjs Rust port). The entire canvas state is a reactive CRDT. There is no "syncing" step—drawing on the board intrinsically modifies the binary state vector, which can instantly compute missing deltas to beam over the wire.
- **Bare-Metal Storage**: Built to persist the canvas straight to disk. High-throughput, local storage speeds with zero serialization lag.
- **ArasDiagram DSL**: Built-in AST and parser for declarative diagramming (`aras-dsl`).
- **Sugiyama Layout**: Beautiful, automatic directed graph layout algorithm built-in (`aras-layout`).

## 📊 Benchmarks

Aerial Engine is built to handle massive architectural diagrams while maintaining a locked **120 FPS**.
By rendering everything into a GPU-accelerated raster cache at the WASM boundary, it solves the traditional `<foreignObject>` SVG rendering bottleneck in Chromium.

**Reproducing the Benchmark:**
We provide a reproducible benchmark script to verify our claims. From the `aerial-engine` directory, run:
```bash
cargo bench --features "benchmark"
```

## 🚀 Getting Started

### Prerequisites
- Node.js & npm
- Rust (`rustup default stable`)
- WebAssembly target: `rustup target add wasm32-unknown-unknown`
- `wasm-pack`

### Build the Engine
From a fresh clone of this repository, you can compile the WASM core for the web target:
```bash
cd aerial-engine
wasm-pack build --target web
```
The output `pkg` folder will contain the raw `.wasm` file and the bindings, which you can drop into any React/Vite project.

## ⚖️ License

Aerial Engine is open-source under the **AGPL-3.0 License**.

If you wish to use Aerial Engine in a closed-source, proprietary, or commercial product without open-sourcing your own code, you must purchase a Commercial License. Please contact `licensing@araskova-labs.com` for details.

See the [LICENSE](LICENSE) file for the full AGPL-3.0 text.

---
*The open core of Aerial. Built by [Araskova Labs](https://araskova.com).*
