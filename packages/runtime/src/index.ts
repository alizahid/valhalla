import "./events";
import "./hmr";

export { createRoot, type Root } from "./host";
export {
    Image,
    Pressable,
    ScrollView,
    Svg,
    Text,
    TextInput,
    View,
    type ClickEvent,
    type ImageProps,
    type KeyEvent,
    type PressableProps,
    type ScrollViewProps,
    type SvgProps,
    type SvgSize,
    type TextInputProps,
    type TextProps,
    type ViewProps,
} from "./components";

// Re-export React itself so consumers can use one import root if they want.
export { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
