//! GPUI app + rquickjs runtime bootstrap.
//!
//! Wires the bundle into a JS context, the scene tree into a GPUI entity, and
//! the two together via host functions (JS → Rust) and the dispatch path
//! (Rust → JS).

use std::sync::Arc;

use anyhow::Context as _;
use gpui::{
    prelude::*, px, size, App as GpuiApp, AppContext, Application, Bounds, Context, Window,
    WindowBounds, WindowOptions,
};
use parking_lot::Mutex;
use rquickjs::{Context as JsContext, Function, Runtime as JsRuntime};

use crate::events::HandlerTable;
use crate::loader::{Bundle, ResolvedBundle};
use crate::scene::{Op, SceneTree};

/// Shared state held between JS host functions and the GPUI entity.
///
/// `Mutex<Vec<Op>>` is the inbox: JS calls `__host_commit` on whatever thread
/// rquickjs happens to run on, but GPUI applies them on the main thread.
pub(crate) struct Bridge {
    pub(crate) inbox: Mutex<Vec<Op>>,
    #[allow(dead_code)] // wired up when free_handlers usage lands
    pub(crate) handlers: HandlerTable,
}

impl Bridge {
    fn new() -> Self {
        Self {
            inbox: Mutex::new(Vec::new()),
            handlers: HandlerTable::new(),
        }
    }
}

/// The GPUI entity that owns the scene tree.
pub struct RootView {
    pub(crate) scene: SceneTree,
    pub(crate) bridge: Arc<Bridge>,
    pub(crate) js: Arc<JsHost>,
}

impl RootView {
    /// Drain pending ops from the inbox and apply them. Called both right
    /// after eval (to pick up the initial mount) and after every dispatch.
    pub(crate) fn drain(&mut self, cx: &mut Context<Self>) {
        let mut inbox = self.bridge.inbox.lock();
        if inbox.is_empty() {
            return;
        }
        for op in inbox.drain(..) {
            self.scene.apply(op);
        }
        cx.notify();
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        crate::render::render_root(self)
    }
}

/// Owns the rquickjs runtime and context. We keep them in `Arc` so the
/// dispatch path can fire JS calls from within the GPUI listener.
pub struct JsHost {
    pub(crate) runtime: JsRuntime,
    pub(crate) context: JsContext,
}

impl JsHost {
    pub fn new() -> anyhow::Result<Self> {
        let runtime = JsRuntime::new().context("creating rquickjs runtime")?;
        let context = JsContext::full(&runtime).context("creating rquickjs context")?;
        Ok(Self { runtime, context })
    }
}

/// Boot a GPUI application, evaluate the bundle, and run.
pub(crate) fn launch(
    title: String,
    bundle: Bundle,
    window_size: (f32, f32),
) -> anyhow::Result<()> {
    let resolved = bundle.resolve()?;
    let bridge = Arc::new(Bridge::new());
    let js = Arc::new(JsHost::new()?);

    install_host_functions(&js, &bridge)?;
    eval_bundle(&js, resolved, &bridge)?;

    Application::new().run(move |cx: &mut GpuiApp| {
        let bounds = Bounds::centered(None, size(px(window_size.0), px(window_size.1)), cx);
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(title.clone().into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let bridge_for_view = bridge.clone();
        let js_for_view = js.clone();

        let _window = cx
            .open_window(opts, move |_window, cx| {
                cx.new(|cx_view: &mut Context<RootView>| {
                    let mut view = RootView {
                        scene: SceneTree::new(),
                        bridge: bridge_for_view.clone(),
                        js: js_for_view.clone(),
                    };
                    view.drain(cx_view);
                    view
                })
            })
            .expect("opening valhalla window");
    });

    Ok(())
}

fn install_host_functions(js: &Arc<JsHost>, bridge: &Arc<Bridge>) -> anyhow::Result<()> {
    let ctx = js.context.clone();
    let bridge_commit = bridge.clone();
    ctx.with(|ctx| -> rquickjs::Result<()> {
        let globals = ctx.globals();

        // __host_commit(opsJson) — JS hands a JSON array of ops per React commit.
        let commit_fn = Function::new(ctx.clone(), move |ops_json: String| {
            match serde_json::from_str::<Vec<Op>>(&ops_json) {
                Ok(ops) => bridge_commit.inbox.lock().extend(ops),
                Err(err) => {
                    log::warn!("[valhalla] bad commit payload: {}", err);
                }
            }
        })?;
        globals.set("__host_commit", commit_fn)?;

        // __host_log(message) — console shim. Plain text from JS to Rust stdout.
        let log_fn = Function::new(ctx.clone(), |message: String| {
            log::info!("[js] {}", message);
        })?;
        globals.set("__host_log", log_fn)?;

        Ok(())
    })?;
    Ok(())
}

fn eval_bundle(
    js: &Arc<JsHost>,
    resolved: ResolvedBundle,
    bridge: &Arc<Bridge>,
) -> anyhow::Result<()> {
    let _ = bridge; // used in the Live arm below
    match resolved {
        ResolvedBundle::Source(src) => {
            js.context.with(|ctx| -> rquickjs::Result<()> {
                ctx.eval::<(), _>(src.as_bytes())?;
                Ok(())
            })?;
            // Pump any pending microtasks (Promises, react-reconciler scheduler).
            while js.runtime.is_job_pending() {
                js.runtime
                    .execute_pending_job()
                    .map_err(|e| anyhow::anyhow!("pending job error: {:?}", e))?;
            }
            Ok(())
        }
        #[cfg(feature = "dev-loader")]
        ResolvedBundle::Live(_session) => {
            // V1: just evaluate a tiny shim that reports we're in dev mode.
            // The HTTP module loader and HMR pump land in V2 — see plan.
            let shim = r#"
                __host_log("[valhalla] dev-mode bundle entry not yet implemented; shipping production build for now.");
            "#;
            js.context.with(|ctx| -> rquickjs::Result<()> {
                ctx.eval::<(), _>(shim.as_bytes())?;
                Ok(())
            })?;
            Ok(())
        }
    }
}
