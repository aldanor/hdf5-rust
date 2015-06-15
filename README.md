# hdf5-rs

[![Build Status](https://img.shields.io/travis/aldanor/hdf5-rs.svg)](https://travis-ci.org/aldanor/hdf5-rs)

Thread-safe Rust bindings and high-level wrappers for the HDF5 library API.

Note that this project is in its early development stage and hence things are likely to change
and break on a regular basis.

## Building

### Windows

Building hdf5-rs on Windows currently needs some manual preparation steps:

* A HDF5 binary, namely the shared library `hdf5.dll` is needed for linking. For (currently) unknown reasons, the prebuilt binaries from [HDF-Group](http://www.hdfgroup.org/) do not work. It has to be build with gcc. Instructions for building HDF5 on Windows can be found [here](http://www.hdfgroup.org/HDF5/release/cmakebuild.html). For building the [TDM distribution](http://tdm-gcc.tdragon.net/)of MinGW-GCC is recommended, as it contains bintools for both 32bit & 64bit.

* Set the environment variable `HDF5_LIBDIR` to point to the folder with the newly build `hdf5.dll`. Explanation: `pkg-config` will silently fail if not present and the path from the before-mentioned environment variable is added to the rustc commands by cargo. (Hint: Avoid path names with spaces, as they are difficult to escape correctly).

* Run `cargo build` and/or `cargo test` to build and rust library and run the tests, repsectively. Hint: After changing the build environment, e.g. `HDF5_LIBDIR`, a `cargo clean` might be necessary.

* Make sure `hdf5.dll` is on your search path, otherwise the tests will fail.


## License

`hdf5-rs` is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details.
