// Vite plugin for Valhalla.
//
// Two responsibilities:
//
// 1. **Build mode** (`vite build`): produce a single self-contained IIFE bundle
//    targeting ES2020, with `process.env.NODE_ENV` baked in. The output drops
//    into `dist/bundle.js` for the Rust crate to `include_str!` or read at
//    startup.
//
// 2. **Dev mode** (`vite dev`): mostly a no-op — `@vitejs/plugin-react` and
//    Vite's normal dev server do the heavy lifting. Our Rust runtime fetches
//    modules over HTTP from the dev server. We just configure entry / target
//    so the served output is shaped correctly.
//
// The plugin is intentionally minimal: it does not implement Fast Refresh
// itself. That comes from `@vitejs/plugin-react`, which the consumer adds
// alongside this plugin.

import type { Plugin, UserConfig } from "vite";

export type ValhallaPluginOptions = {
    /**
     * Entry module. Defaults to `src/main.tsx`.
     */
    entry?: string;
    /**
     * IIFE global name used in build output. Defaults to `ValhallaApp`.
     */
    globalName?: string;
};

export default function valhalla(options: ValhallaPluginOptions = {}): Plugin {
    const entry = options.entry ?? "src/main.tsx";
    const globalName = options.globalName ?? "ValhallaApp";

    return {
        name: "@valhalla/vite",

        config(_userConfig, env): UserConfig {
            // Dev: rely on the existing dev server. Just make sure the entry
            // gets pre-bundled.
            if (env.command === "serve") {
                return {
                    optimizeDeps: {
                        include: ["react", "react/jsx-runtime", "react-reconciler"],
                    },
                };
            }

            // Build: library mode with IIFE output.
            return {
                build: {
                    target: "es2020",
                    lib: {
                        entry,
                        name: globalName,
                        formats: ["iife"],
                        fileName: () => "bundle.js",
                    },
                    minify: false,
                    sourcemap: "inline",
                    rollupOptions: {
                        // No externals — bundle everything React-shaped into the
                        // IIFE so the embedded JS engine has it all in one shot.
                        external: [],
                        output: {
                            inlineDynamicImports: true,
                        },
                    },
                },
                define: {
                    "process.env.NODE_ENV": JSON.stringify(
                        process.env.NODE_ENV ?? "production",
                    ),
                },
            };
        },

        // Inject a tiny runtime bootstrap into HTML (dev mode). This is what
        // the Rust loader sees at the dev-server root. It does nothing
        // browser-specific; it just imports the user's entry, which in turn
        // imports `@valhalla/runtime` and calls `createRoot().render(...)`.
        transformIndexHtml() {
            return [
                {
                    tag: "script",
                    attrs: { type: "module", src: `/${entry}` },
                    injectTo: "body",
                },
            ];
        },
    };
}
