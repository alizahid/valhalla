//! Valhalla — React for the desktop, rendered through GPUI.
//!
//! See the workspace README for the architecture overview. The crate exposes a
//! tiny `App` builder; the Vite plugin and runtime npm packages are what
//! consumers use day-to-day.

use std::path::PathBuf;

pub use anyhow;
pub use gpui;

mod assets;
mod events;
mod input;
mod loader;
mod render;
mod runtime;
mod scene;
mod style;
mod tailwind;
mod widgets;

/// Re-exports for the headless smoke-test binary. Not part of the public API.
#[doc(hidden)]
pub mod __internal_for_headless {
    pub use crate::scene::{ElementProps, Node, NodeId, Op, SceneTree};
}

pub use loader::Bundle;

/// Top-level builder. Mirrors the public API documented in the plan.
pub struct App {
    title: String,
    bundle: Bundle,
    window_size: (f32, f32),
    assets_dir: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            title: "Valhalla".into(),
            bundle: Bundle::Auto,
            window_size: (800.0, 600.0),
            assets_dir: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn bundle(mut self, bundle: Bundle) -> Self {
        self.bundle = bundle;
        self
    }

    pub fn window_size(mut self, width: f32, height: f32) -> Self {
        self.window_size = (width, height);
        self
    }

    /// Filesystem directory the framework joins with relative `src` props on
    /// `<Svg>` / `<Image>` / `<Button icon="…" />`. Without this, relative
    /// paths resolve against the current working directory.
    pub fn assets_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.assets_dir = Some(dir.into());
        self
    }

    pub fn run(self) -> anyhow::Result<()> {
        if let Some(dir) = self.assets_dir {
            assets::set_assets_dir(dir);
        }
        runtime::launch(self.title, self.bundle, self.window_size)
    }
}

/// Compile-time bundle embed: `Bundle::Embedded(include_str!("dist/bundle.js"))`
/// scoped to the consumer crate's manifest dir.
#[macro_export]
macro_rules! embed_bundle {
    () => {
        $crate::Bundle::Embedded(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/dist/bundle.js"
        )))
    };
    ($path:literal) => {
        $crate::Bundle::Embedded(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)))
    };
}

/// Resolves `Bundle::Auto` based on the `VALHALLA_DEV` env var.
pub(crate) fn resolve_auto_bundle() -> Bundle {
    if let Ok(url) = std::env::var("VALHALLA_DEV") {
        Bundle::Vite { url }
    } else if let Ok(path) = std::env::var("VALHALLA_BUNDLE") {
        Bundle::File(PathBuf::from(path))
    } else {
        // Fall back to a tiny "no app loaded" bundle so the window still opens.
        Bundle::Embedded(NO_APP_BUNDLE)
    }
}

const NO_APP_BUNDLE: &str = r#"
__host_log("[valhalla] no bundle configured. Set VALHALLA_DEV=http://localhost:5173 or VALHALLA_BUNDLE=/path/to/bundle.js, or call .bundle() explicitly.");
"#;
