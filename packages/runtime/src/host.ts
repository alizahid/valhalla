// React reconciler HostConfig.
//
// Mutation mode. We never build a JS-side DOM — instead, every mutation method
// pushes a small op into a buffer, and `resetAfterCommit` flushes the buffer
// to Rust as one JSON array. One IPC per React commit.
//
// The ops protocol matches `crates/valhalla/src/scene.rs::Op`. Field shapes
// come from there; if you change a name on one side, change it on both.

import Reconciler from "react-reconciler";
import { DefaultEventPriority } from "react-reconciler/constants.js";
import { freeHandler, registerHandler } from "./events";

export type Instance = {
    id: number;
    type: string;
    props: any;
    handlers: Record<string, number>;
};

export type TextInstance = {
    id: number;
    type: "#text";
};

type AnyInstance = Instance | TextInstance;

let nextNodeId = 1;
let pendingOps: any[] = [];
let currentUpdatePriority = 0;

function pushOp(op: any) {
    pendingOps.push(op);
}

// Build a fresh handler map for a node, freeing any handlers it previously
// held that aren't in the new prop set.
function syncHandlers(
    inst: Instance,
    props: any,
): Record<string, number> {
    const next: Record<string, number> = {};
    for (const key in props) {
        if (key.startsWith("on") && typeof props[key] === "function") {
            next[key] = registerHandler(props[key]);
        }
    }
    for (const key in inst.handlers) {
        if (next[key] === undefined) {
            freeHandler(inst.handlers[key]);
        } else if (next[key] !== inst.handlers[key]) {
            // Replaced — free old.
            freeHandler(inst.handlers[key]);
        }
    }
    inst.handlers = next;
    return next;
}

function freeAllHandlers(inst: Instance) {
    for (const key in inst.handlers) {
        freeHandler(inst.handlers[key]);
    }
    inst.handlers = {};
}

function extractClasses(props: any): string[] {
    const v = props.className;
    if (typeof v !== "string" || v.length === 0) return [];
    return v.split(/\s+/).filter(Boolean);
}

function extractStyle(props: any): Record<string, unknown> {
    const s = props.style;
    if (!s || typeof s !== "object") return {};
    return s;
}

// Anything that isn't a known structural prop, an event handler, or a
// non-serializable value gets shipped to Rust as an attribute. Widgets pull
// the keys they care about; the rest are ignored on the Rust side.
const STRUCTURAL_KEYS = new Set([
    "children",
    "className",
    "style",
    "key",
    "ref",
]);

function extractAttrs(props: any): Record<string, unknown> {
    const out: Record<string, unknown> = {};
    for (const key in props) {
        if (STRUCTURAL_KEYS.has(key)) continue;
        if (key.startsWith("on")) continue;
        const v = props[key];
        if (v === undefined || v === null) continue;
        const t = typeof v;
        if (t === "function") continue;
        if (t === "string" || t === "number" || t === "boolean") {
            out[key] = v;
        }
        // Plain objects / arrays are skipped — widgets that need richer attrs
        // can be added on a per-tag basis later.
    }
    return out;
}

function setPropsOp(inst: Instance, props: any) {
    const handlers = syncHandlers(inst, props);
    pushOp({
        op: "SetProps",
        id: inst.id,
        classes: extractClasses(props),
        style: extractStyle(props),
        handlers,
        attrs: extractAttrs(props),
    });
}

const hostConfig: any = {
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: true,
    noTimeout: -1,
    supportsMicrotasks: true,
    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    scheduleMicrotask:
        typeof queueMicrotask === "function"
            ? queueMicrotask
            : (cb: () => void) => Promise.resolve().then(cb),

    getRootHostContext() {
        return null;
    },
    getChildHostContext(parentCtx: any) {
        return parentCtx;
    },
    getCurrentEventPriority() {
        return DefaultEventPriority;
    },

    // React 19 added these update-priority hooks alongside event priority.
    // We don't have a real concept of priority here, so a single shared
    // variable is enough to keep the reconciler happy.
    resolveUpdatePriority() {
        return currentUpdatePriority || DefaultEventPriority;
    },
    setCurrentUpdatePriority(p: number) {
        currentUpdatePriority = p;
    },
    getCurrentUpdatePriority() {
        return currentUpdatePriority;
    },

    getPublicInstance(inst: AnyInstance) {
        return inst;
    },

    maySuspendCommit() {
        return false;
    },

    prepareForCommit() {
        return null;
    },
    resetAfterCommit() {
        if (pendingOps.length === 0) return;
        const json = JSON.stringify(pendingOps);
        pendingOps = [];
        try {
            __host_commit(json);
        } catch (err) {
            __host_log(
                `[valhalla] __host_commit threw: ${(err as Error)?.message ?? err}`,
            );
        }
    },

    shouldSetTextContent() {
        return false;
    },
    resetTextContent() {
        /* no-op: we never put text into element nodes directly */
    },

    createInstance(type: string, props: any): Instance {
        const id = nextNodeId++;
        pushOp({ op: "Create", id, tag: type });
        const inst: Instance = { id, type, props, handlers: {} };
        setPropsOp(inst, props);
        return inst;
    },
    createTextInstance(text: string): TextInstance {
        const id = nextNodeId++;
        pushOp({ op: "CreateText", id, value: text });
        return { id, type: "#text" };
    },

    appendInitialChild(parent: AnyInstance, child: AnyInstance) {
        pushOp({ op: "Append", parent: parent.id, child: child.id });
    },
    finalizeInitialChildren() {
        return false;
    },

    appendChild(parent: AnyInstance, child: AnyInstance) {
        pushOp({ op: "Append", parent: parent.id, child: child.id });
    },
    appendChildToContainer(_container: any, child: AnyInstance) {
        pushOp({ op: "Append", parent: 0, child: child.id });
    },
    insertBefore(parent: AnyInstance, child: AnyInstance, before: AnyInstance) {
        pushOp({
            op: "Insert",
            parent: parent.id,
            child: child.id,
            before: before.id,
        });
    },
    insertInContainerBefore(_container: any, child: AnyInstance, before: AnyInstance) {
        pushOp({ op: "Insert", parent: 0, child: child.id, before: before.id });
    },
    removeChild(parent: AnyInstance, child: AnyInstance) {
        pushOp({ op: "Remove", parent: parent.id, child: child.id });
        if ("handlers" in (child as Instance)) {
            freeAllHandlers(child as Instance);
        }
        pushOp({ op: "Drop", id: child.id });
    },
    removeChildFromContainer(_container: any, child: AnyInstance) {
        pushOp({ op: "Remove", parent: 0, child: child.id });
        if ("handlers" in (child as Instance)) {
            freeAllHandlers(child as Instance);
        }
        pushOp({ op: "Drop", id: child.id });
    },

    commitUpdate(inst: Instance, _type: string, _prev: any, next: any) {
        inst.props = next;
        setPropsOp(inst, next);
    },
    commitTextUpdate(inst: TextInstance, _prev: string, next: string) {
        pushOp({ op: "UpdateText", id: inst.id, value: next });
    },

    clearContainer(_container: any) {
        // The Rust side doesn't currently expose a "clear root" op — it does
        // the right thing on the next SetRoot. We could add a Clear op later.
    },

    preparePortalMount() {
        /* portals not supported yet */
    },
    detachDeletedInstance() {
        /* no-op, see removeChild */
    },
};

const reconciler = (Reconciler as any)(hostConfig);

const CONTAINER_TOKEN = { __valhallaContainer: true };

let cachedContainer: any = null;
function getContainer() {
    if (!cachedContainer) {
        cachedContainer = reconciler.createContainer(
            CONTAINER_TOKEN,
            0, // LegacyRoot — keeps things simple for the demo
            null,
            false,
            null,
            "",
            (err: unknown) => __host_log(`[valhalla] recoverable error: ${err}`),
            null,
        );
    }
    return cachedContainer;
}

export type Root = {
    render(element: any): void;
    unmount(): void;
};

export function createRoot(): Root {
    const container = getContainer();
    return {
        render(element) {
            reconciler.updateContainer(element, container, null, null);
        },
        unmount() {
            reconciler.updateContainer(null, container, null, null);
        },
    };
}
