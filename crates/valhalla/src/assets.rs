//! Asset path resolution.
//!
//! Source paths from JS are interpreted relative to the app's configured
//! assets directory. Absolute paths are passed through unchanged so users
//! who already manage their own paths (or who use Vite's static handling)
//! can point at anything they want.
//!
//! The framework deliberately does not bundle any icons or images of its
//! own — userland decides which set to ship.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ASSETS_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Set the global assets root. Called once during `App::run()` when the
/// builder configured `.assets_dir(...)`. Subsequent calls are ignored —
/// changing the root mid-run has no defined semantics.
pub fn set_assets_dir(dir: PathBuf) {
    let _ = ASSETS_DIR.set(dir);
}

pub fn get_assets_dir() -> Option<&'static PathBuf> {
    ASSETS_DIR.get()
}

/// Resolve a `src` from JS into an absolute filesystem path.
///
/// - Absolute paths pass through.
/// - Relative paths join with the configured assets dir.
/// - Without a configured dir, relative paths join with the current
///   working directory — friendly default for `cargo run` from a project
///   root.
pub fn resolve(src: &str) -> PathBuf {
    let p = Path::new(src);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let trimmed = src.trim_start_matches('/');
    match ASSETS_DIR.get() {
        Some(dir) => dir.join(trimmed),
        None => std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(trimmed),
    }
}
