// `import.meta.hot` polyfill, lightweight.
//
// In a browser, Vite injects an HMR client that opens a WebSocket and exposes
// `import.meta.hot` per module. In our environment, the WebSocket lives in
// Rust (see `crates/valhalla/src/loader/vite.rs`); incoming HMR messages
// arrive here via `globalThis.__host_hmr`.
//
// This file is intentionally small. Full Fast Refresh requires the Vite
// plugin's per-module injection of `react-refresh` registration; we expose
// the minimal API surface that injection expects (`accept`, `dispose`,
// `data`) so the wired-up version can land without further runtime changes.

type AcceptCb = (mod: any) => void;
type DisposeCb = (data: any) => void;

type HotEntry = {
    accept: AcceptCb[];
    dispose: DisposeCb[];
    data: any;
};

const registry = new Map<string, HotEntry>();

function entry(id: string): HotEntry {
    let e = registry.get(id);
    if (!e) {
        e = { accept: [], dispose: [], data: {} };
        registry.set(id, e);
    }
    return e;
}

export type Hot = {
    accept(cb?: AcceptCb): void;
    accept(deps: string[], cb: AcceptCb): void;
    dispose(cb: DisposeCb): void;
    invalidate(): void;
    data: any;
    on(event: string, cb: (...args: any[]) => void): void;
    off(event: string, cb: (...args: any[]) => void): void;
};

export function makeHot(id: string): Hot {
    const e = entry(id);
    const listeners = new Map<string, Set<(...args: any[]) => void>>();

    return {
        accept(_a?: any, _b?: any) {
            // We accept either form; until react-refresh wiring lands, the
            // host-driven full reload covers both.
            if (typeof _a === "function") {
                e.accept.push(_a as AcceptCb);
            } else if (Array.isArray(_a) && typeof _b === "function") {
                e.accept.push(_b as AcceptCb);
            }
        },
        dispose(cb: DisposeCb) {
            e.dispose.push(cb);
        },
        invalidate() {
            // Tell the host to do a full reload via the standard message.
            (globalThis as any).__host_hmr_request_full_reload?.();
        },
        get data() {
            return e.data;
        },
        on(ev, cb) {
            let set = listeners.get(ev);
            if (!set) {
                set = new Set();
                listeners.set(ev, set);
            }
            set.add(cb);
        },
        off(ev, cb) {
            listeners.get(ev)?.delete(cb);
        },
    };
}

// The Rust side calls this with each HMR frame from Vite. We currently only
// react to `full-reload` — anything more granular waits until the loader
// can re-fetch and re-eval modules in place.
(globalThis as any).__host_hmr = (frameJson: string) => {
    let frame: any;
    try {
        frame = JSON.parse(frameJson);
    } catch {
        return;
    }
    switch (frame.type) {
        case "connected":
            __host_log("[valhalla] HMR connected");
            return;
        case "full-reload":
            __host_log("[valhalla] HMR full-reload");
            return;
        case "update":
            __host_log("[valhalla] HMR update (V1: full reload required)");
            return;
        case "error":
            __host_log(`[valhalla] HMR error: ${JSON.stringify(frame.err)}`);
            return;
        default:
            return;
    }
};
