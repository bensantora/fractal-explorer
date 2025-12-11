// Load the wasm-bindgen bootstrap (default export)
// and the Rust-exposed functions (named exports).
import init, { init as wasm_init, zoom_at } from "./pkg/fractal_explorer.js";

let wasmReady = false;

async function run() {
    // Load and initialize the WASM module
    await init();

    // Initialize the Rust fractal engine with the canvas ID
    await wasm_init("fractal-canvas");

    wasmReady = true;

    const canvas = document.getElementById("fractal-canvas");

    // Handle click-to-zoom
    canvas.addEventListener("click", (event) => {
        if (!wasmReady) return;

        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;

        zoom_at(x, y, 2.0);
    });
}

run();
