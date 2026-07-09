use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // No image assets, but Bundle::Auto resolves the JS bundle relative to
    // this directory: <assets_dir>/../dist/bundle.js.
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    valhalla::App::new()
        .title("Calculator")
        .bundle(valhalla::Bundle::Auto)
        .window_size(340.0, 560.0)
        .assets_dir(assets)
        .run()
}
