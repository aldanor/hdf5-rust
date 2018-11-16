use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest_rs::Config::default();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.target_rustcflags = Some("-Ltarget/debug/ -Ltarget/debug/deps/".to_owned());
    config.mode = cfg_mode;
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    compiletest_rs::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail");
}
