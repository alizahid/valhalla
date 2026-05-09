// React-Native-shaped primitives. Each one is a thin wrapper that maps to a
// host element name the Rust render walk dispatches on. No hidden behaviour
// in the wrapper — everything observable goes through the JSX → mutation-op
// pipeline.
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
// Generic flex container. Maps to GPUI `div`. Use for layout.

export type ViewProps = CommonProps & {
    onKeyDown?: (event: KeyEvent) => void;
    onKeyUp?: (event: KeyEvent) => void;
};

export function View(props: ViewProps) {
    return createElement("view", props);
}

// ─── Text ────────────────────────────────────────────────────────────────
// Text container. Children are strings or further inline content. Styling
// (color, size, weight) applies to the contained text.

export type TextProps = CommonProps & {
    numberOfLines?: number;
};

export function Text(props: TextProps) {
    return createElement("text", props);
}

// ─── Pressable ───────────────────────────────────────────────────────────
// Clickable wrapper. Same as View but with `onPress`.

export type PressableProps = CommonProps & {
    onPress?: (event: ClickEvent) => void;
    disabled?: boolean;
};

export function Pressable(props: PressableProps) {
    return createElement("pressable", props);
}

// ─── Button ──────────────────────────────────────────────────────────────
// gpui-component Button. Variants follow shadcn/ui conventions.

export type ButtonVariant =
    | "primary"
    | "secondary"
    | "ghost"
    | "outline"
    | "danger"
    | "link";

export type ButtonSize = "xs" | "sm" | "md" | "lg";

export type ButtonProps = {
    label?: string;
    children?: ReactNode;
    variant?: ButtonVariant;
    size?: ButtonSize;
    disabled?: boolean;
    onPress?: (event: ClickEvent) => void;
    className?: string;
    style?: CSSProperties;
};

export function Button(props: ButtonProps) {
    return createElement("button", props);
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
// Single-line text input.

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

// ─── Checkbox ────────────────────────────────────────────────────────────
// gpui-component Checkbox.

export type CheckboxProps = {
    checked: boolean;
    onValueChange?: (value: boolean) => void;
    label?: string;
    disabled?: boolean;
    className?: string;
    style?: CSSProperties;
};

export function Checkbox(props: CheckboxProps) {
    return createElement("checkbox", props);
}

// ─── Switch ──────────────────────────────────────────────────────────────
// gpui-component Switch.

export type SwitchProps = {
    checked: boolean;
    onValueChange?: (value: boolean) => void;
    label?: string;
    disabled?: boolean;
    className?: string;
    style?: CSSProperties;
};

export function Switch(props: SwitchProps) {
    return createElement("switch", props);
}

// ─── Divider ─────────────────────────────────────────────────────────────
// Horizontal or vertical separator.

export type DividerProps = {
    vertical?: boolean;
    label?: string;
    className?: string;
    style?: CSSProperties;
};

export function Divider(props: DividerProps) {
    return createElement("divider", props);
}

// ─── Badge ───────────────────────────────────────────────────────────────
// Small colored label. Implemented as a styled View; users can also build
// their own with className alone.

export type BadgeProps = CommonProps & {
    variant?: "default" | "success" | "warning" | "danger" | "info";
};

export function Badge(props: BadgeProps) {
    return createElement("badge", props);
}
