// Public entry. `createRoot` mirrors react-dom; the rest are typed primitives
// the consumer imports directly.

import "./events";
import "./hmr";

export { createRoot, type Root } from "./host";
export {
    Badge,
    Button,
    Checkbox,
    Divider,
    Image,
    Pressable,
    ScrollView,
    Svg,
    Switch,
    Text,
    TextInput,
    View,
    type BadgeProps,
    type ButtonProps,
    type ButtonSize,
    type ButtonVariant,
    type CheckboxProps,
    type ClickEvent,
    type DividerProps,
    type ImageProps,
    type KeyEvent,
    type PressableProps,
    type ScrollViewProps,
    type SvgProps,
    type SvgSize,
    type SwitchProps,
    type TextInputProps,
    type TextProps,
    type ViewProps,
} from "./components";

// Re-export React itself so consumers can use one import root if they want.
// Their own `react` install is what wins at bundle time — this is mostly a
// convenience for `import { useState } from "@valhalla/runtime"`.
export { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
