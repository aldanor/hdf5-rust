extern crate hdf5_derive;
use hdf5_derive::H5Type;

#[derive(H5Type)]
//~^ ERROR proc-macro derive
//~^^ HELP Cannot derive H5Type for empty enums
enum Foo {}

fn main() {}
