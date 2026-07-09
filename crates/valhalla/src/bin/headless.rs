//! Smoke test: load a bundle in rquickjs, capture every `__host_commit` it
//! emits, print the resulting scene tree. No GPUI involvement — this lets
//! us validate the JS bridge without booting a window.
//!
//! Run with:
//!
//! ```sh
//! cargo run --bin valhalla-headless -- path/to/bundle.js
//! ```
//!
//! Optionally simulate clicks after mount. Each `--press LABEL` finds the
//! first element (DFS order) that has an `onPress`/`onClick` handler and
//! whose subtree text equals LABEL, dispatches the handler into JS, and
//! applies the resulting ops. This drives real user flows end-to-end:
//!
//! ```sh
//! cargo run --bin valhalla-headless -- examples/calculator/dist/bundle.js \
//!     --press 7 --press + --press 8 --press =
//! # scene tree now shows "15" in the display
//! ```

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

struct Cli {
    bundle: PathBuf,
    presses: Vec<String>,
    quiet_ops: bool,
}

fn parse_cli() -> anyhow::Result<Cli> {
    let mut bundle = None;
    let mut presses = Vec::new();
    let mut quiet_ops = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--press" => {
                let label = args.next().context("--press requires a LABEL argument")?;
                presses.push(label);
            }
            "--quiet-ops" => quiet_ops = true,
            _ if bundle.is_none() => bundle = Some(PathBuf::from(arg)),
            other => anyhow::bail!("unexpected argument: {}", other),
        }
    }
    Ok(Cli {
        bundle: bundle
            .context("usage: valhalla-headless <bundle.js> [--quiet-ops] [--press LABEL]...")?,
        presses,
        quiet_ops,
    })
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = parse_cli()?;
    let source = std::fs::read_to_string(&cli.bundle)
        .with_context(|| format!("reading bundle from {}", cli.bundle.display()))?;

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

    drain_jobs(&runtime)?;

    let captured_ops = std::mem::take(&mut *inbox.lock().unwrap());
    let captured_logs = std::mem::take(&mut *logs.lock().unwrap());

    println!("=== logs ({} lines) ===", captured_logs.len());
    for line in &captured_logs {
        println!("  {}", line);
    }

    println!("=== ops ({} total) ===", captured_ops.len());
    if !cli.quiet_ops {
        for op in &captured_ops {
            println!("  {:?}", op);
        }
    }

    let mut scene = SceneTree::new();
    for op in captured_ops {
        scene.apply(op);
    }
    println!("=== scene root: {:?} ===", scene.root());
    if let Some(root_id) = scene.root() {
        print_tree(&scene, root_id, 0);
    }

    // Simulated presses. Each one is looked up fresh against the current
    // scene (handler IDs churn across re-renders, so labels are the stable
    // way to address a button).
    for label in &cli.presses {
        let handler = find_press_handler(&scene, label)
            .with_context(|| format!("no pressable element with text {:?} found", label))?;
        println!("\n=== press {:?} (handler {}) ===", label, handler);
        ctx.with(|ctx| -> rquickjs::Result<()> {
            let dispatch: Function = ctx.globals().get("__dispatchEvent")?;
            let _: () = dispatch.call((handler, "{}".to_string()))?;
            Ok(())
        })?;
        drain_jobs(&runtime)?;

        let ops = std::mem::take(&mut *inbox.lock().unwrap());
        println!("=== ops after press ({} total) ===", ops.len());
        if !cli.quiet_ops {
            for op in &ops {
                println!("  {:?}", op);
            }
        }
        for op in ops {
            scene.apply(op);
        }
    }

    if !cli.presses.is_empty() {
        if let Some(root_id) = scene.root() {
            println!("\n=== final scene ===");
            print_tree(&scene, root_id, 0);
        }
    }

    Ok(())
}

fn drain_jobs(runtime: &JsRuntime) -> anyhow::Result<()> {
    while runtime.is_job_pending() {
        runtime
            .execute_pending_job()
            .map_err(|e| anyhow::anyhow!("pending job error: {:?}", e))?;
    }
    Ok(())
}

/// Find the handler for the element best matching `label`. Exact subtree
/// text match wins; otherwise the pressable with the shortest subtree text
/// that *contains* the label (most specific ancestor — e.g. a suggestion
/// row whose corner badge says "Add").
fn find_press_handler(scene: &SceneTree, label: &str) -> Option<u32> {
    let root = scene.root()?;
    let mut exact: Option<u32> = None;
    let mut containing: Option<(usize, u32)> = None;
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if let Some(Node::Element {
            props, children, ..
        }) = scene.get(id)
        {
            let handler = props
                .handlers
                .get("onPress")
                .or_else(|| props.handlers.get("onClick"));
            if let Some(&h) = handler {
                let text = subtree_text(scene, id);
                let text = text.trim();
                if text == label && exact.is_none() {
                    exact = Some(h);
                } else if text.contains(label)
                    && containing.map_or(true, |(len, _)| text.len() < len)
                {
                    containing = Some((text.len(), h));
                }
            }
            // Push in reverse so DFS visits children in document order.
            for &c in children.iter().rev() {
                stack.push(c);
            }
        }
    }
    exact.or(containing.map(|(_, h)| h))
}

fn subtree_text(scene: &SceneTree, id: NodeId) -> String {
    let mut out = String::new();
    let mut stack = vec![id];
    while let Some(id) = stack.pop() {
        match scene.get(id) {
            Some(Node::Text { value }) => out.push_str(value),
            Some(Node::Element { children, .. }) => {
                for &c in children.iter().rev() {
                    stack.push(c);
                }
            }
            None => {}
        }
    }
    out
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
