# hdf5-rs

[![Build Status](https://img.shields.io/travis/aldanor/hdf5-rs.svg)](https://travis-ci.org/aldanor/hdf5-rs) [![Appveyor Build Status](https://img.shields.io/appveyor/ci/aldanor/hdf5-rs.svg)](https://ci.appveyor.com/project/aldanor/hdf5-rs)

[Documentation](http://aldanor.github.io/hdf5-rs/hdf5_rs)

Thread-safe Rust bindings and high-level wrappers for the HDF5 library API.

Requires HDF5 library of version 1.8.4 or later.

## Compatibility

### Platforms

`hdf5-rs` is known to run on these platforms:

- Linux (tested on Travis CI, HDF5 v1.8.4)
- OS X (tested on Travis CI, HDF5 v1.8.16)
- Windows (see below for details; gnu build tested on AppVeyor, HDF5 v1.8.15)

### Rust

`hdf5-rs` is tested for all three official release channels: stable, beta and nightly.

## Building

### Conditional compilation

Build scripts for both `libhdf5-sys` and `hdf5-rs` crates check the actual version of the
HDF5 library that they are being linked against, and some functionality may be conditionally
enabled or disabled at compile time. While this allows supporting multiple versions of HDF5
in a single codebase, this is something the library user should be aware of in case they
choose to use the low level FFI bindings.

### Linux, OS X

There are also two environment variables that may be of use if the library location and/or name
is unconventional:

- `HDF5_LIBDIR` – added to library search path during the build step
- `HDF5_LIBNAME` – library filename (defaults to `hdf5`)

Note also that the build script of `libhdf5-sys` crate tries to use `pkg-config` (if it's available
to deduce library location).

For most setups though, just running `cargo build` and `cargo test` should be sufficient.

### Windows

Until the official MSVC tooling lands becomes mature enough in stable Rust, we can only support the
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
