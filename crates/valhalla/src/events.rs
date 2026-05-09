//! Event dispatch.
//!
//! React's event handlers live in JS as plain functions. We never serialize
//! them; instead, the runtime package allocates an opaque integer id per
//! handler and stores `id → fn` in JS. Rust receives the id via the prop
//! payload and stores it on the scene node. When GPUI fires (click, keydown),
//! we call back into JS with `__dispatchEvent(id, payloadJson)`.

use std::sync::Arc;

use anyhow::Context as _;
use parking_lot::Mutex;
use rquickjs::{Function, Value};

use crate::runtime::JsHost;

/// In V1 there's no Rust-side bookkeeping per id — the registry lives in JS.
/// This struct is a placeholder for future ref counting / leak detection.
#[derive(Default)]
pub struct HandlerTable {
    _live: Mutex<()>,
}

impl HandlerTable {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Synchronously dispatch one event into JS and pump any microtasks the
/// callback enqueued (React schedules updates via `queueMicrotask`).
///
/// `__host_commit` is normally invoked synchronously from React's commit
/// phase, which runs inside `dispatchEvent` → so by the time this returns,
/// the inbox already contains any new ops the handler caused.
pub fn dispatch(
    js: &Arc<JsHost>,
    handler_id: u32,
    payload: serde_json::Value,
) -> anyhow::Result<()> {
    let payload_json = payload.to_string();

    js.context.with(|ctx| -> anyhow::Result<()> {
        let globals = ctx.globals();
        let dispatch_fn: Value = globals
            .get("__dispatchEvent")
            .context("__dispatchEvent not registered yet (bundle not loaded?)")?;
        let dispatch_fn = dispatch_fn
            .into_function()
            .context("__dispatchEvent is not a function")?;
        let _: () = dispatch_fn
            .call((handler_id, payload_json))
            .context("invoking __dispatchEvent")?;
        Ok(())
    })?;

    while js.runtime.is_job_pending() {
        js.runtime
            .execute_pending_job()
            .map_err(|e| anyhow::anyhow!("pending job error: {:?}", e))?;
    }

    Ok(())
}

/// Drop the handler-id storage on the JS side. Called when a node is removed
/// or its handler set changed. JS-side `freeHandler` is idempotent.
#[allow(dead_code)] // wired in V2 when render::removeChild walks descendants
pub fn free_handlers(js: &Arc<JsHost>, ids: &[u32]) {
    if ids.is_empty() {
        return;
    }
    let _ = js.context.with(|ctx| -> rquickjs::Result<()> {
        let globals = ctx.globals();
        if let Ok(free_fn) = globals.get::<_, Function>("__host_free_handlers") {
            let payload = serde_json::to_string(ids).unwrap_or_else(|_| "[]".into());
            let _: rquickjs::Result<()> = free_fn.call((payload,));
        }
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::HandlerTable;

    #[test]
    fn handler_table_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HandlerTable>();
    }
}
