# Valhalla

> React for the desktop. Native pixels via GPUI, JavaScript via QuickJS, Tailwind for styling.

## What this is

Valhalla is a proof-of-concept "React Native, but desktop." You write React + Tailwind in TypeScript; it renders through Zed's [GPUI](https://www.gpui.rs/) framework as a native, GPU-accelerated desktop app. There's no Electron, no webview, no DOM.

The user-facing surface is small:

```rust
// my-app/src/main.rs
fn main() -> anyhow::Result<()> {
    valhalla::App::new()
        .title("My App")
        .bundle(valhalla::Bundle::Auto)
        .run()
}
```

```tsx
// my-app/src/App.tsx
import { useState } from "react";
import { TextInput } from "@valhalla/runtime";

export default function App() {
    const [n, setN] = useState(0);
    return (
        <div className="flex flex-col items-center justify-center gap-4 p-8 size-full bg-slate-900">
            <div className="text-3xl text-white">Count: {n}</div>
            <button className="px-4 py-2 rounded bg-blue-500 text-white"
                    onClick={() => setN(n + 1)}>+</button>
        </div>
    );
}
```

## Architecture

```
┌──────────────── valhalla crate (in user's binary) ──────────────────┐
│                                                                       │
│  ┌─────────────────────┐   ops    ┌───────────────────────────────┐  │
│  │ rquickjs Context    │  (JSON)  │ GPUI App                      │  │
│  │                     │          │                               │  │
│  │ React 19            │─────────▶│ RootView (Entity)             │  │
│  │ react-reconciler    │          │ ├─ scene: SceneTree           │  │
│  │ HostConfig          │          │ ├─ render() walks tree        │  │
│  │ user app code       │◀─────────│ └─ on_click → JS              │  │
│  │ idToFn map          │  events  │                               │  │
│  └─────────────────────┘          │ tailwind::parse + style::apply│  │
│                                   └───────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

The canonical scene tree lives in **Rust**. JS owns no DOM — `react-reconciler` emits mutation ops, they cross as one JSON array per React commit, Rust applies them and GPUI re-renders.

### Why QuickJS instead of Hermes

The user originally suggested Hermes. Research showed no production-grade Rust embedding exists in May 2026 (`hermes_rs` is a bytecode disassembler; `rusty_hermes` is unreleased; upstream Hermes only exposes the C++ JSI, not a stable C ABI). [`rquickjs`](https://github.com/DelSkayn/rquickjs) is the swap — full ES2020, ~210 KiB engine, plain `cargo build`, ergonomic Rust↔JS via derive macros. React 19 runs as-is.

## Repo layout

```
crates/valhalla/         publishable Rust crate (the runtime)
packages/runtime/        @valhalla/runtime — HostConfig, components, hooks
packages/vite-plugin/    @valhalla/vite — Vite plugin for build + dev
examples/counter/        the demo app (Cargo bin + React)
```

## What works today

The headless smoke test loads the bundle, runs React, captures every op:

```
$ cd examples/counter && bun run build
$ cd ../.. && cargo run -p valhalla --bin valhalla-headless -- examples/counter/dist/bundle.js
```

This validates, end-to-end:

- Vite produces a single ES2020 IIFE bundle (1.3 MB) including React 19, react-reconciler, our `@valhalla/runtime`, and the user's `App.tsx`.
- rquickjs evaluates the bundle (with a tiny browser-globals shim for `setTimeout`, `console`, etc.).
- React renders `<App />`. The reconciler walks the tree.
- Our HostConfig pushes mutation ops into a buffer, flushes once per commit via `__host_commit`.
- Rust deserializes ops into typed `Op` values.
- `SceneTree::apply` builds the mirror tree.
- Synthetic dispatch: calling `__dispatchEvent(handlerId, payload)` from Rust fires the React handler, which calls `setState`, which causes a re-render, which produces new ops back into the inbox. The `Count: 0` text becomes `Count: 1`.

The `counter` binary itself (`cargo run -p counter`) compiles against `gpui` and `gpui-component` and opens a window — that part can't be visually verified in CI but the type contract is right.

## What's stubbed (next up)

Per the [plan](.claude/plans/i-wanna-see-if-fluffy-beacon.md), V1 promises Fast Refresh, full keyboard input, and a working `<input>` field. Those land in subsequent commits:

- **Fast Refresh / HMR.** The Rust loader's WebSocket bridge (`crates/valhalla/src/loader/vite.rs`) connects to Vite's HMR socket and forwards frames to JS, but the JS-side `import.meta.hot` polyfill (`packages/runtime/src/hmr.ts`) isn't yet wired into `react-refresh` — it currently just logs frames.
- **`<input>` editing.** Renders the current `value` / `placeholder` and accepts `onClick`, but doesn't yet integrate `gpui-component::input::TextField` for cursor/typing/IME. Pure rendering of the prop bundle is correct.
- **Keyboard events.** `onKeyDown` / `onKeyUp` are routed through the prop bundle to Rust but the GPUI focus + key dispatch wiring isn't enabled yet.
- **`free_handlers`** is implemented but not yet called on `removeChild` — handler IDs leak across re-renders for now.

These are scoped V1 features and the design is laid out; the foundation under them (the React→Rust pipeline) is real and tested.

## Toolchain

- Rust 1.94+
- Bun 1.3+
- `libxkbcommon-dev`, `libxkbcommon-x11-dev` (Linux only — GPUI links against them)
