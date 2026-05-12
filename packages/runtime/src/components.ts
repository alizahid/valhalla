// Framework primitives — only what gpui itself provides.
//
// View, Text, Pressable, ScrollView, Svg, Image, TextInput. That's the
// entire library surface. Buttons / Checkboxes / Switches / Badges /
// Dividers / Modals / Tooltips are userland concerns — compose them
// from the primitives below. Look at `examples/kanban/src/App.tsx` for
// a worked example.
//
// Tag names are lowercase strings. React treats lowercase JSX as host
// elements (the renderer's responsibility) and PascalCase as components.
// We export PascalCase functions that emit lowercase host elements.

import { createElement, type CSSProperties, type ReactNode } from "react";

// ─── Shared types ─────────────────────────────────────────────────────────

export type KeyEvent = {
    key: string;
    code: string;
    ctrlKey: boolean;
    shiftKey: boolean;
    altKey: boolean;
    metaKey: boolean;
    repeat: boolean;
};

export type ClickEvent = {
    x: number;
    y: number;
};

type CommonProps = {
    children?: ReactNode;
    className?: string;
    style?: CSSProperties;
};

// ─── View ────────────────────────────────────────────────────────────────
// Generic flex container. Maps to GPUI `div`.

export type ViewProps = CommonProps & {
    onClick?: (event: ClickEvent) => void;
    onKeyDown?: (event: KeyEvent) => void;
    onKeyUp?: (event: KeyEvent) => void;
};

export function View(props: ViewProps) {
    return createElement("view", props);
}

// ─── Text ────────────────────────────────────────────────────────────────
// Text container. Children are strings or further inline content.
// Styling (color, size, weight) applies to the contained text.

export type TextProps = CommonProps & {
    numberOfLines?: number;
};

export function Text(props: TextProps) {
    return createElement("text", props);
}

// ─── Pressable ───────────────────────────────────────────────────────────
// Clickable wrapper. Build buttons / pressable cards / list rows on top.

export type PressableProps = CommonProps & {
    onPress?: (event: ClickEvent) => void;
    disabled?: boolean;
};

export function Pressable(props: PressableProps) {
    return createElement("pressable", props);
}

// ─── ScrollView ──────────────────────────────────────────────────────────
// Scrollable region. Vertical by default; pass `horizontal` for horizontal.

export type ScrollViewProps = CommonProps & {
    horizontal?: boolean;
};

export function ScrollView(props: ScrollViewProps) {
    return createElement("scrollview", props);
}

// ─── TextInput ───────────────────────────────────────────────────────────
// Single-line text input. V1 stub — renders as a div showing the current
// value / placeholder, no editing yet.

export type TextInputProps = {
    value: string;
    onChange?: (event: { target: { value: string } }) => void;
    onChangeText?: (text: string) => void;
    placeholder?: string;
    className?: string;
    style?: CSSProperties;
    autoFocus?: boolean;
    disabled?: boolean;
    onKeyDown?: (event: KeyEvent) => void;
    onKeyUp?: (event: KeyEvent) => void;
    onFocus?: () => void;
    onBlur?: () => void;
};

export function TextInput(props: TextInputProps) {
    return createElement("textinput", props);
}

// ─── Svg ─────────────────────────────────────────────────────────────────
// Vector image. `src` is a path resolved against the app's assets
// directory (set via App.assets_dir() in Rust) or an absolute path.
// `tint` applies a fill color via gpui's `text_color()` on the SVG.

export type SvgSize = "xs" | "sm" | "md" | "lg" | number;

export type SvgProps = {
    src: string;
    size?: SvgSize;
    /** CSS color string. Sets gpui's `text_color` on the SVG, which acts
     * as the fill / stroke (depends on the SVG using `currentColor`). */
    tint?: string;
    className?: string;
    style?: CSSProperties;
};

export function Svg(props: SvgProps) {
    return createElement("svg", props);
}

// ─── Image ───────────────────────────────────────────────────────────────
// Raster image. Same path-resolution rules as Svg.

export type ImageProps = {
    src: string;
    width?: number;
    height?: number;
    /** Mirrors React Native's `resizeMode`. */
    objectFit?: "contain" | "cover" | "fill" | "none";
    className?: string;
    style?: CSSProperties;
};

export function Image(props: ImageProps) {
    return createElement("image", props);
}
