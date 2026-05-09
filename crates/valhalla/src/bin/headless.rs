//! Smoke test: load a bundle in rquickjs, capture every `__host_commit` it
//! emits, print the resulting scene tree. No GPUI involvement — this lets
//! us validate the JS bridge without booting a window.
//!
//! Run with: `cargo run --bin valhalla-headless -- path/to/bundle.js`.

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use rquickjs::{
    CatchResultExt, CaughtError, Context as JsContext, Function, Runtime as JsRuntime,
};

use valhalla::__internal_for_headless::{Node, NodeId, Op, SceneTree};

fn install_browser_shims<'js>(ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
    // The React bundle expects `setTimeout`, `clearTimeout`, `queueMicrotask`,
    // and a `console` object with at least `log`/`warn`/`error`. rquickjs
    // doesn't ship them — easiest path is a tiny JS prelude that wires them
    // up to host functions or queues that the runtime's pending-job loop
    // already drains.
    let prelude = r#"
        (function() {
            const microtasks = [];
            globalThis.queueMicrotask = (cb) => {
                Promise.resolve().then(cb);
            };
            // Best-effort timeout: just runs the callback in the next
            // microtask. Good enough for React's scheduler in a smoke test.
            globalThis.setTimeout = (cb, _ms) => {
                Promise.resolve().then(cb);
                return 0;
            };
            globalThis.clearTimeout = () => {};
            globalThis.setInterval = (cb, _ms) => 0;
            globalThis.clearInterval = () => {};
            const stringify = (v) => {
                try { return typeof v === 'string' ? v : JSON.stringify(v); }
                catch { return String(v); }
            };
            const fmt = (args) => Array.from(args).map(stringify).join(' ');
            globalThis.console = {
                log:   (...args) => __host_log(fmt(args)),
                info:  (...args) => __host_log(fmt(args)),
                warn:  (...args) => __host_log('[warn] '  + fmt(args)),
                error: (...args) => __host_log('[error] ' + fmt(args)),
                debug: (...args) => __host_log('[debug] ' + fmt(args)),
            };
        })();
    "#;
    ctx.eval::<(), _>(prelude.as_bytes())?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: valhalla-headless <bundle.js>")?;
    let source = std::fs::read_to_string(&path)
        .with_context(|| format!("reading bundle from {}", path.display()))?;

    let runtime = JsRuntime::new()?;
    let ctx = JsContext::full(&runtime)?;

    let inbox: Arc<Mutex<Vec<Op>>> = Arc::new(Mutex::new(Vec::new()));
    let logs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let inbox_for_cb = inbox.clone();
    let logs_for_cb = logs.clone();

    ctx.with(|ctx| -> anyhow::Result<()> {
        let globals = ctx.globals();
        let commit = Function::new(ctx.clone(), move |ops_json: String| {
            match serde_json::from_str::<Vec<Op>>(&ops_json) {
                Ok(ops) => inbox_for_cb.lock().unwrap().extend(ops),
                Err(err) => eprintln!("[headless] bad commit: {} :: {}", err, ops_json),
            }
        })?;
        globals.set("__host_commit", commit)?;

        let log = Function::new(ctx.clone(), move |msg: String| {
            logs_for_cb.lock().unwrap().push(msg);
        })?;
        globals.set("__host_log", log)?;

        // Browser-ish globals the React bundle expects.
        install_browser_shims(&ctx)?;

        match ctx.eval::<(), _>(source.as_bytes()).catch(&ctx) {
            Ok(()) => Ok(()),
            Err(CaughtError::Exception(exc)) => {
                let msg = exc.message().unwrap_or_default();
                let stack = exc.stack().unwrap_or_default();
                anyhow::bail!("JS exception: {}\n{}", msg, stack);
            }
            Err(other) => anyhow::bail!("JS error: {:?}", other),
        }
    })?;

    while runtime.is_job_pending() {
        runtime
            .execute_pending_job()
            .map_err(|e| anyhow::anyhow!("pending job error: {:?}", e))?;
    }

    let captured_ops = std::mem::take(&mut *inbox.lock().unwrap());
    let captured_logs = std::mem::take(&mut *logs.lock().unwrap());

    println!("=== logs ({} lines) ===", captured_logs.len());
    for line in &captured_logs {
        println!("  {}", line);
    }

    println!("=== ops ({} total) ===", captured_ops.len());
    for op in &captured_ops {
        println!("  {:?}", op);
    }

    let mut scene = SceneTree::new();
    for op in captured_ops {
        scene.apply(op);
    }
    println!("=== scene root: {:?} ===", scene.root());
    if let Some(root_id) = scene.root() {
        print_tree(&scene, root_id, 0);
    }

    // Synthetic click on handler 2 (the "+" button per the demo). We expect
    // React to setState, re-render, and re-emit ops with the count text now
    // showing 1 instead of 0.
    println!("\n=== dispatching click on handler 2 (the + button) ===");
    ctx.with(|ctx| -> rquickjs::Result<()> {
        let dispatch: Function = ctx.globals().get("__dispatchEvent")?;
        let _: () = dispatch.call((2u32, "{}".to_string()))?;
        Ok(())
    })?;
    while runtime.is_job_pending() {
        runtime
            .execute_pending_job()
            .map_err(|e| anyhow::anyhow!("pending job error: {:?}", e))?;
    }

    let after_click = std::mem::take(&mut *inbox.lock().unwrap());
    println!("=== ops after click ({} total) ===", after_click.len());
    for op in &after_click {
        println!("  {:?}", op);
    }
    for op in after_click {
        scene.apply(op);
    }
    if let Some(root_id) = scene.root() {
        println!("\n=== scene after click ===");
        print_tree(&scene, root_id, 0);
    }

    Ok(())
}

fn print_tree(scene: &SceneTree, id: NodeId, depth: usize) {
    let pad = "  ".repeat(depth);
    match scene.get(id) {
        Some(Node::Text { value }) => println!("{}#{}  text {:?}", pad, id, value),
        Some(Node::Element {
            tag,
            props,
            children,
        }) => {
            println!(
                "{}#{}  <{}> classes={:?} handlers={:?} attrs={:?}",
                pad, id, tag, props.classes, props.handlers, props.attrs
            );
            for &c in children {
                print_tree(scene, c, depth + 1);
            }
        }
        None => println!("{}#{} <missing>", pad, id),
    }
}
