#[macro_use]
extern crate hdf5_derive;

use std::marker::PhantomData;

#[derive(H5Type)]
//~^ ERROR proc-macro derive
//~^^ HELP Cannot derive H5Type for empty tuple structs
struct Foo<T>(PhantomData<T>);
