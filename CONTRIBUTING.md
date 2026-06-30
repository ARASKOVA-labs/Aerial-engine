# Contributing to Aerial

First off, thank you for considering contributing to Aerial. It's people like you that make Aerial a powerful tool for everyone. 

Aerial is built on a highly optimized, pure Rust WebAssembly engine wrapped in a React/Tauri interface. We welcome contributions across all layers of the stack!

## How to Contribute

### 1. Reporting Bugs
- Ensure the bug was not already reported by searching on GitHub under **Issues**.
- If you're unable to find an open issue addressing the problem, open a new one. Be sure to include a title and clear description, as much relevant information as possible, and a code sample or an executable test case demonstrating the expected behavior that is not occurring.

### 2. Suggesting Enhancements
- Open a new issue with the `enhancement` label.
- Clearly describe the feature you would like, why you need it, and how it should work.

### 3. Submitting Pull Requests
- **Fork the repository** and clone it locally.
- **Create a branch** (`git checkout -b feature/your-feature-name`).
- **Make your changes**. 
    - If you are changing the Rust engine, ensure you run `wasm-pack build` and that `cargo test` passes.
    - If you are changing the UI, ensure your changes match the premium, minimalist design system of the app.
- **Commit your changes** (`git commit -m 'Add some feature'`).
- **Push to the branch** (`git push origin feature/your-feature-name`).
- **Open a Pull Request** against the `main` branch.

## Local Development Setup

To develop Aerial locally, you will need:
1. **Rust:** Latest stable version.
2. **Bun:** The fast JavaScript runtime.
3. **Tauri CLI:** Installed globally or run via `bunx`.

### Quickstart
```bash
# Install dependencies
bun install

# Build the WASM engine (Requires wasm-pack)
cd aerial-core/aerial-engine
wasm-pack build --target web --out-dir ../../public/aerial-engine --no-typescript
cd ../../

# Run the Tauri development server
bun run tauri dev
```

## Architectural Guidelines
- **Zero-Copy Rule:** The Rust engine operates on a zero-copy philosophy. Do not introduce unnecessary allocations in the critical render loop.
- **WASM Boundary:** Keep the WASM boundary thin. Pass simple data types and avoid excessive serialization/deserialization between JS and Rust.
- **Design:** Ensure any UI changes adhere strictly to the monochrome/glassmorphism aesthetic. We prefer clean, borderless interfaces with subtle blurs and shadows.

Thank you for contributing to the future of high-performance canvas applications!
