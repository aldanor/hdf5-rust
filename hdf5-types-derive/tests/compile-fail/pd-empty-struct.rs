extern crate hdf5_types;
#[macro_use]
extern crate hdf5_types_derive;

use std::marker::PhantomData;

#[derive(H5Type)]
//~^ ERROR proc-macro derive
//~^^ HELP Cannot derive H5Type for empty structs
struct Foo<T> {
    t: PhantomData<T>,
}
