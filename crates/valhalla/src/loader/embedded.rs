use std::path::Path;

use anyhow::Context;

pub fn read(path: &Path) -> anyhow::Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("loading bundle from {}", path.display()))
}
