extern crate hdf5_derive;
use hdf5_derive::H5Type;

use std::marker::PhantomData;

#[derive(H5Type)]
//~^ ERROR proc-macro derive
//~^^ HELP Cannot derive H5Type for empty tuple structs
struct Foo<T>(PhantomData<T>);

fn main() {}
