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
import { Button, Switch, Text, View } from "@valhalla/runtime";

export default function App() {
    const [enabled, setEnabled] = useState(false);
    const [n, setN] = useState(0);
    return (
        <View className="flex flex-col items-center justify-center gap-4 p-8 size-full bg-slate-900">
            <Text className="text-3xl text-white">Count: {n}</Text>
            <Switch checked={enabled} onValueChange={setEnabled} label="Auto-increment" />
            <Button label="Add one" variant="primary" onPress={() => setN(n + 1)} />
        </View>
    );
}
```

Components are React-Native-style primitives: `View`, `Text`, `Pressable`, `Button`, `TextInput`, `Checkbox`, `Switch`, `ScrollView`, `Divider`, `Badge`. Each maps to a host element name the Rust render walk dispatches on, so adding a new primitive is a TS export + a Rust renderer.

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
examples/kanban/         the demo app (Cargo bin + React)
```

## Primitives

`@valhalla/runtime` exports React-Native-shaped primitives. Each is a tag-based host element the Rust render walk dispatches on:

| Component | Tag | Renderer |
|---|---|---|
| `View` | `view` | GPUI `div` |
| `Text` | `text` | GPUI `div` styled as text |
| `Pressable` | `pressable` | GPUI `div` with `cursor_pointer` + click |
| `Button` | `button` | `gpui_component::Button` (variants: `primary`, `secondary`, `ghost`, `outline`, `danger`, `link`) |
| `TextInput` | `textinput` | (V1: read-only display; gpui-component TextField integration is next) |
| `Checkbox` | `checkbox` | `gpui_component::Checkbox` |
| `Switch` | `switch` | `gpui_component::Switch` |
| `ScrollView` | `scrollview` | GPUI `div.overflow_{x,y}_scroll` |
| `Divider` | `divider` | `gpui_component::Divider` |
| `Badge` | `badge` | Styled `div` with variant palette |
| `Svg` | `svg` | GPUI `svg()` element. Takes `src` (path), optional `size` and `tint`. |
| `Image` | `image` | GPUI `img()` element. Takes `src`, `width`, `height`, `objectFit`. |

**Icons are intentionally userland.** The framework ships `<Svg>` and `<Image>`; the consumer picks an icon set (lucide, heroicons, custom SVGs) and writes a one-line `Icon` wrapper. See `examples/kanban/src/App.tsx`:

```tsx
const iconPath = (name: string) => `icons/${name}.svg`;
function Icon({ name, size }: { name: string; size?: SvgSize }) {
    return <Svg src={iconPath(name)} size={size} />;
}
```

Asset paths resolve against the `assets_dir` configured on the `App` builder:

```rust
valhalla::App::new()
    .assets_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"))
    .run()?
```

Adding a new primitive is a TS export (`createElement("foo", props)`) plus a Rust renderer (`match tag { "foo" => render_foo(...) }`).

## What works today

The headless smoke test loads the bundle, runs React, captures every op:

```
$ cd examples/kanban && bun run build
$ cd ../.. && cargo run -p valhalla --bin valhalla-headless -- examples/kanban/dist/bundle.js
```

The Kanban demo renders into **534 mutation ops** on initial mount — three columns of cards, a header with switches, a detail panel with priority buttons + a checkbox, and a stats badge. Clicking a Switch (e.g. "Sort by priority") dispatches into JS, fires `setState`, re-renders, and emits the diff back as new ops (~143 in that case).

This validates, end-to-end:

- Vite produces a single ES2020 IIFE bundle (~1.4 MB) including React 19, react-reconciler, our `@valhalla/runtime`, and the user's `App.tsx`.
- rquickjs evaluates the bundle (with a tiny browser-globals shim for `setTimeout`, `console`, etc.).
- React renders `<App />`. The reconciler walks the tree.
- Our HostConfig pushes mutation ops into a buffer, flushes once per commit via `__host_commit`.
- Rust deserializes ops into typed `Op` values.
- `SceneTree::apply` builds the mirror tree.
- Synthetic dispatch: calling `__dispatchEvent(handlerId, payload)` from Rust fires the React handler, which calls `setState`, which produces follow-up ops into the inbox.

The `kanban` binary itself (`cargo run -p kanban`) compiles against `gpui` and `gpui-component` and opens a window — that part can't be visually verified in CI but the type contract is right.

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
