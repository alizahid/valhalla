use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Assets live next to the example's source — `assets/icons/*.svg` etc.
    // CARGO_MANIFEST_DIR is the example's own crate directory at compile
    // time, which is what we want for `cargo run -p kanban` from anywhere.
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    valhalla::App::new()
        .title("Kanban")
        .bundle(valhalla::Bundle::Auto)
        .window_size(1100.0, 700.0)
        .assets_dir(assets)
        .run()
}
