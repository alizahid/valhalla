// Globals injected by the Rust host before our bundle is evaluated.
//
// `__host_commit` accepts a JSON-encoded array of mutation ops and applies
// them to the Rust SceneTree. `__host_log` is a console shim that goes to
// the Rust process's stdout. Both are present in every runtime mode.
declare global {
    function __host_commit(opsJson: string): void;
    function __host_log(message: string): void;
    function __host_free_handlers(idsJson: string): void;

    // Set by the runtime at module load; called by Rust on every event.
    var __dispatchEvent: ((handlerId: number, payloadJson: string) => void) | undefined;
}

export {};
