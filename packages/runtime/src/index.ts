// Public entry. `createRoot` mirrors react-dom; the rest are typed primitives
// the consumer imports directly.

import "./events";
import "./hmr";

export { createRoot, type Root } from "./host";
export { TextInput, View, type KeyEvent, type TextInputProps, type ViewProps } from "./components";

// Re-export React itself so consumers can use one import root if they want.
// Their own `react` install is what wins at bundle time — this is mostly a
// convenience for `import { useState } from "@valhalla/runtime"`.
export { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
