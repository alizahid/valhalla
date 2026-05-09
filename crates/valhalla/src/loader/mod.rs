//! Bundle resolution.
//!
//! Three modes covering the lifetime of an app: dev (Vite server with HMR),
//! prod-from-disk (loose file), and prod-embedded (`include_str!`). `Auto`
//! reads env vars to pick at runtime.

use std::path::PathBuf;

mod embedded;

#[cfg(feature = "dev-loader")]
pub mod vite;

/// Where to load JavaScript from.
#[derive(Debug, Clone)]
pub enum Bundle {
    /// Compile-time embed via `include_str!`. Single-binary distribution.
    Embedded(&'static str),

    /// Read from disk at startup. Useful for prod where the bundle ships
    /// next to the binary as a separate asset.
    File(PathBuf),

    /// Connect to a running Vite dev server. Modules are fetched lazily over
    /// HTTP; HMR messages arrive over WebSocket. Requires `dev-loader` feature.
    Vite { url: String },

    /// Inspect env: `VALHALLA_DEV` → Vite, `VALHALLA_BUNDLE` → File, else
    /// fall back to a tiny empty bundle.
    Auto,
}

/// Resolved bundle, ready to feed to the runtime.
pub enum ResolvedBundle {
    /// Synchronous source — eval once at startup.
    Source(String),

    /// Live dev session. Holds a connection to Vite.
    #[cfg(feature = "dev-loader")]
    Live(vite::ViteSession),
}

impl Bundle {
    pub fn resolve(self) -> anyhow::Result<ResolvedBundle> {
        let resolved = match self {
            Bundle::Embedded(s) => Bundle::Embedded(s),
            Bundle::File(p) => Bundle::File(p),
            Bundle::Vite { url } => Bundle::Vite { url },
            Bundle::Auto => crate::resolve_auto_bundle(),
        };

        match resolved {
            Bundle::Embedded(s) => Ok(ResolvedBundle::Source(s.to_string())),
            Bundle::File(p) => Ok(ResolvedBundle::Source(embedded::read(&p)?)),
            #[cfg(feature = "dev-loader")]
            Bundle::Vite { url } => Ok(ResolvedBundle::Live(vite::ViteSession::connect(url)?)),
            #[cfg(not(feature = "dev-loader"))]
            Bundle::Vite { .. } => {
                anyhow::bail!("Bundle::Vite requires the `dev-loader` feature");
            }
            Bundle::Auto => unreachable!("resolved above"),
        }
    }
}
