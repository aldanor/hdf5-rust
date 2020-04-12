extern crate hdf5_derive;
use hdf5_derive::H5Type;

#[derive(H5Type)]
//~^ ERROR proc-macro derive
//~^^ HELP H5Type requires #[repr(C)] for structs
struct Foo(i64);

fn main() {}
