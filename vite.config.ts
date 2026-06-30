import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

  // Serve .wasm files with correct MIME type
  assetsInclude: ['**/*.wasm'],

  // Prevent Vite from obscuring rust errors
  clearScreen: false,

  // Tauri expects a fixed port
  server: {
    port: 1420,
    strictPort: true,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
    watch: {
      ignored: ["**/src-tauri/**", "**/aerial-engine/target/**"],
    },
  },
}));
