extern crate hdf5_types;
#[macro_use]
extern crate hdf5_types_derive;

#[derive(H5Type)]
//~^ ERROR proc-macro derive
//~^^ HELP H5Type requires #[repr(C)] for structs
struct Foo { bar: i64 }
