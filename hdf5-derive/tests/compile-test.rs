use std::fs::{read_dir, remove_file};

// Workaround for https://github.com/laumann/compiletest-rs/issues/114
#[allow(dead_code)]
fn clean_rlibs(config: &compiletest_rs::Config) {
    if config.target_rustcflags.is_some() {
        for directory in config.target_rustcflags.as_ref().unwrap().split_whitespace() {
            if let Ok(mut entries) = read_dir(directory) {
                while let Some(Ok(entry)) = entries.next() {
                    let f = entry.file_name().clone().into_string().unwrap();
                    if f.ends_with(".rmeta") {
                        let prefix = &f[..f.len() - 5];
                        let _ = remove_file(entry.path());
                        if let Ok(mut entries) = read_dir(directory) {
                            while let Some(Ok(entry)) = entries.next() {
                                let f = entry.file_name().clone().into_string().unwrap();
                                if f.starts_with(prefix) && !f.ends_with(".rmeta") {
                                    let _ = remove_file(entry.path());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn run_mode(mode: &'static str) {
    let mut config = compiletest_rs::Config::default();
    let cfg_mode = mode.parse().expect("Invalid mode");

    config.mode = cfg_mode;
    config.src_base = format!("tests/{}", mode).into();
    config.verbose = true;
    config.link_deps();
    // clean_rlibs(&config);  // commented out for now as it's flaky on CI

    compiletest_rs::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail");
}
