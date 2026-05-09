fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    valhalla::App::new()
        .title("Counter")
        .bundle(valhalla::Bundle::Auto)
        .window_size(480.0, 360.0)
        .run()
}
