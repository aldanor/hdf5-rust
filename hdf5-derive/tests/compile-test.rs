fn run_mode(mode: &'static str) {
    let mut config = compiletest_rs::Config::default();
    let cfg_mode = mode.parse().expect("Invalid mode");

    config.mode = cfg_mode;
    config.src_base = format!("tests/{}", mode).into();
    config.verbose = true;
    config.link_deps();
    config.clean_rmeta();

    compiletest_rs::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail");
}
