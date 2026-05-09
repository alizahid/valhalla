// Typed primitives users can import. They're thin wrappers over the host
// element names — `<TextInput>` is just `<input>` with React's keyboard
// event types pre-baked.

import { createElement, type CSSProperties, type ReactNode } from "react";

export type ViewProps = {
    children?: ReactNode;
    className?: string;
    style?: CSSProperties;
    onClick?: (event: { x: number; y: number }) => void;
    onKeyDown?: (event: KeyEvent) => void;
    onKeyUp?: (event: KeyEvent) => void;
};

export type KeyEvent = {
    key: string;
    code: string;
    ctrlKey: boolean;
    shiftKey: boolean;
    altKey: boolean;
    metaKey: boolean;
    repeat: boolean;
};

/** A plain container — equivalent to `<div>`, but typed for the desktop event shape. */
export function View(props: ViewProps) {
    return createElement("div", props);
}

export type TextInputProps = {
    value: string;
    onChange?: (event: { target: { value: string } }) => void;
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

/** Single-line text input. V1: ASCII-only, no IME / clipboard / selection. */
export function TextInput(props: TextInputProps) {
    return createElement("input", props);
}
