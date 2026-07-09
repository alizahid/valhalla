import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import valhalla from "@valhalla/vite";

export default defineConfig({
    plugins: [react(), valhalla()],
});
