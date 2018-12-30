use super::gen::gen_ascii;

pub fn random_filename() -> String {
    gen_ascii(&mut rand::thread_rng(), 8)
}

pub fn new_in_memory_file() -> h5::Result<h5::File> {
    let filename = random_filename();
    h5::File::with_options().mode("w").driver("core").filebacked(false).open(&filename)
}
