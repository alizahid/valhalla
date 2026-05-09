// JS-side handler registry.
//
// Functions never cross the bridge — we keep them here, hand the host an
// opaque integer per `(node, eventName)`, and the host calls back through
// `__dispatchEvent` when GPUI fires.

const idToFn = new Map<number, (event: any) => void>();
let nextHandlerId = 1;

export function registerHandler(fn: (event: any) => void): number {
    const id = nextHandlerId++;
    idToFn.set(id, fn);
    return id;
}

export function freeHandler(id: number): void {
    idToFn.delete(id);
}

export function freeHandlers(ids: number[]): void {
    for (const id of ids) idToFn.delete(id);
}

// Wire the dispatch entry point. The host calls this with the handler id and
// a JSON payload (object with shape depending on the event type — `{}` for
// click, `{ key, code, ctrlKey, ... }` for keydown, etc.).
(globalThis as any).__dispatchEvent = (handlerId: number, payloadJson: string): void => {
    const fn = idToFn.get(handlerId);
    if (!fn) return;
    let payload: any;
    try {
        payload = JSON.parse(payloadJson);
    } catch {
        payload = {};
    }
    try {
        fn(payload);
    } catch (err) {
        try {
            __host_log(`[valhalla] handler ${handlerId} threw: ${(err as Error)?.message ?? err}`);
        } catch {
            /* unreachable */
        }
    }
};

// Mirror image: let the host free a batch of ids when nodes drop. Rust calls
// this from `removeChild` propagation.
(globalThis as any).__host_free_handlers = (idsJson: string): void => {
    try {
        const ids = JSON.parse(idsJson) as number[];
        freeHandlers(ids);
    } catch {
        /* swallow — log shim may not be installed yet */
    }
};
