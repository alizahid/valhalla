import { createRoot } from "@valhalla/runtime";
import App from "./App";

// Expose the bootstrap on a global so the Rust runtime can call it after the
// bundle finishes evaluating. In dev mode this script runs at module load,
// so the IIFE form below also works as a side-effecting entry.
function start() {
    const root = createRoot();
    root.render(<App />);
}

(globalThis as any).ValhallaApp = { start };
start();
