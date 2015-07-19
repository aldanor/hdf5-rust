# hdf5-rs

[![Build Status](https://img.shields.io/travis/aldanor/hdf5-rs.svg)](https://travis-ci.org/aldanor/hdf5-rs) [![Appveyor Build Status](https://img.shields.io/appveyor/ci/aldanor/hdf5-rs.svg)](https://ci.appveyor.com/project/aldanor/hdf5-rs)

[Documentation](http://aldanor.github.io/hdf5-rs)

Thread-safe Rust bindings and high-level wrappers for the HDF5 library API.

## Compatibility

### Platforms

`hdf5-rs` is known to run on these platforms:

- Linux (tests run on Travis CI)
- OS X
- Windows (MinGW only for now, see below for details; tests run on AppVeyor)

### Rust

`hdf5-rs` is tested for all three official release channels:

- stable (1.1.0)
- beta
- nightly

## Building

### Linux, OS X

There are also two environment variables that may be of use if the library location and/or name
is unconventional:

- `HDF5_LIBDIR` -- added to library search path during the build step
- `HDF5_LIBNAME` -- library filename (defaults to `hdf5`)

Note also that the build script of `libhdf5-sys` crate tries to use `pkg-config` (if it's available
to deduce library location).

For most setups though, just running `cargo build` and `cargo test` should be sufficient.

### Windows

Until the official MSVC tooling lands in stable Rust (presumably in 1.2.0), we can only support the
gcc build of HDF5 binaries on Windows. Since the official binaries from
[HDF-Group](http://www.hdfgroup.org/) are built with MSVC, a few extra step are required to get
everything working. Instructions for building HDF5 on Windows can be found
[here](http://www.hdfgroup.org/HDF5/release/cmakebuild.html). The
[TDM distribution](http://tdm-gcc.tdragon.net/) of MinGW-GCC is recommended as it contains bintools
for both 32-bit and 64-bit.

As of now, building `hdf5-rs` on Windows requires a few manual steps:

- gcc-compatible HDF5 binary (`hdf5.dll` shared library) is required for linking.

- Set `HDF5_LIBDIR` environment variable to point to the folder containing `hdf5.dll` (avoid paths
  with spaces as they are difficult to escape correctly).

- Make sure that `hdf5.dll` is on your search path, otherwise the tests will fail.

- Run `cargo build` and/or `cargo test` to build the Rust library and run the tests. After making
  any changes to the build environment, e.g. `HDF5_LIBDIR`, a `cargo clean` may be necessary.

## License

`hdf5-rs` is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details.
